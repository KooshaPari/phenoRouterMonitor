//! Async trait helpers for Phenotype ecosystem.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// AsyncIterator trait - async version of the standard Iterator.
pub trait AsyncIterator {
    type Item;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
    fn size_hint(&self) -> (usize, Option<usize>) { (0, None) }
}

impl<T: AsyncIterator + Unpin> AsyncIterator for &mut T {
    type Item = T::Item;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut *self).poll_next(cx)
    }
    fn size_hint(&self) -> (usize, Option<usize>) { (**self).size_hint() }
}

/// Extension trait for AsyncIterator with utilities.
pub trait AsyncIteratorExt: AsyncIterator {
    fn collect_vec(self) -> CollectVec<Self> where Self: Sized { CollectVec::new(self) }
}

impl<T: AsyncIterator> AsyncIteratorExt for T {}

/// Collector that accumulates items into a vector.
pub struct CollectVec<I: AsyncIterator> { iterator: I, items: Vec<I::Item> }

impl<I: AsyncIterator> CollectVec<I> { fn new(iterator: I) -> Self { Self { iterator, items: Vec::new() } } }
impl<I: AsyncIterator + Unpin> Unpin for CollectVec<I> {}

impl<I: AsyncIterator + Unpin> AsyncIterator for CollectVec<I> {
    type Item = I::Item;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match Pin::new(&mut this.iterator).poll_next(cx) {
            Poll::Ready(Some(item)) => { this.items.push(item); Poll::Pending }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.items.len();
        let upper = self.iterator.size_hint().1.map(|s| s + len);
        (len, upper)
    }
}

impl<I: AsyncIterator> CollectVec<I> { pub fn into_vec(self) -> Vec<I::Item> { self.items } }

/// Wrapper for boxed async futures.
pub struct AsyncFuture<T> { inner: Pin<Box<dyn Future<Output = T> + Send>> }

impl<T: Send + 'static> AsyncFuture<T> {
    pub fn new<F>(future: F) -> Self where F: Future<Output = T> + Send + 'static { 
        Self { inner: Box::pin(future) } 
    }
    
    pub fn map<U: 'static>(self, f: impl FnOnce(T) -> U + Send + 'static) -> AsyncFuture<U> {
        let inner = self.inner;
        AsyncFuture { inner: Box::pin(async move { f(inner.await) }) }
    }
}

impl<T: Send + 'static, E: Send + 'static> AsyncFuture<Result<T, E>> {
    pub fn ok(self) -> AsyncFuture<Option<T>> {
        self.map(|r| r.ok())
    }
    pub fn err(self) -> AsyncFuture<Option<E>> {
        self.map(|r| r.err())
    }
}

impl<T: Send + 'static> Future for AsyncFuture<T> {
    type Output = T;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> { 
        self.inner.as_mut().poll(cx) 
    }
}

/// Trait for types that need async cleanup.
pub trait AsyncDrop { fn async_drop(self); }

/// Wrapper providing AsyncDrop for types with cleanup closures.
pub struct AsyncDropper<T> {
    value: Option<T>,
    cleanup: Option<Box<dyn FnOnce(T) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + 'static>>,
}

impl<T: Send + 'static> AsyncDropper<T> {
    pub fn new<F>(value: T, cleanup: F) -> Self 
    where F: FnOnce(T) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + 'static {
        Self { value: Some(value), cleanup: Some(Box::new(cleanup)) }
    }
}

impl<T: Send + 'static> AsyncDrop for AsyncDropper<T> {
    fn async_drop(mut self) {
        if let (Some(value), Some(cleanup)) = (self.value.take(), self.cleanup.take()) {
            let _ = cleanup(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_async_future_map() {
        let future = AsyncFuture::new(async { 42u32 });
        let mapped = future.map(|v| v * 2);
        assert_eq!(mapped.await, 84);
    }

    #[test]
    fn test_async_dropper() {
        static CALLED: AtomicUsize = AtomicUsize::new(0);
        struct TestValue(i32);
        { 
            let dropper = AsyncDropper::new(TestValue(42), |val| Box::pin(async move { 
                CALLED.store(val.0 as usize, Ordering::SeqCst); 
            })); 
            dropper.async_drop(); 
        }
        assert_eq!(CALLED.load(Ordering::SeqCst), 42);
    }

    #[tokio::test]
    async fn test_async_future_new() {
        let future = AsyncFuture::new(async { "hello" });
        assert_eq!(future.await, "hello");
    }

    #[tokio::test]
    async fn test_async_future_result_ok() {
        let ok_future = AsyncFuture::new(async { Ok::<u32, &str>(42) });
        let ok_fut = ok_future.ok();
        assert_eq!(ok_fut.await, Some(42));
    }

    #[tokio::test]
    async fn test_async_future_result_err() {
        let err_future = AsyncFuture::new(async { Err::<u32, &str>("error") });
        let err_fut = err_future.err();
        assert_eq!(err_fut.await, Some("error"));
    }
}
