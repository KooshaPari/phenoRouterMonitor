//! Async trait helpers for Phenotype ecosystem.
//!
//! This crate provides async versions of common std traits and utilities
//! for working with async/await in Rust.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// AsyncIterator trait - async version of the standard Iterator.
pub trait AsyncIterator {
    type Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
    fn size_hint(&self) -> (usize, Option<usize>) { (0, None) }
}

impl<T: AsyncIterator> AsyncIterator for &mut T {
    type Item = T::Item;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(*self).poll_next(cx)
    }
    fn size_hint(&self) -> (usize, Option<usize>) { (**self).size_hint() }
}

/// AsyncIterator extension trait with utilities.
pub trait AsyncIteratorExt: AsyncIterator {
    fn collect_vec(self) -> CollectVec<Self> where Self: Sized { CollectVec::new(self) }
}

impl<T: AsyncIterator> AsyncIteratorExt for T {}

/// Collector that accumulates items into a vector.
pub struct CollectVec<I: AsyncIterator> { iterator: I, items: Vec<I::Item> }

impl<I: AsyncIterator> CollectVec<I> {
    fn new(iterator: I) -> Self { Self { iterator, items: Vec::new() } }
}

impl<I: AsyncIterator + Unpin> Unpin for CollectVec<I> {}

impl<I: AsyncIterator> AsyncIterator for CollectVec<I> {
    type Item = I::Item;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.iterator).poll_next(cx) {
            Poll::Ready(Some(item)) => { self.items.push(item); Poll::Pending }
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

impl<I: AsyncIterator> CollectVec<I> {
    pub fn into_vec(self) -> Vec<I::Item> { self.items }
}

/// Wrapper for boxed async futures.
pub struct AsyncFuture<T: Send> {
    inner: Pin<Box<dyn Future<Output = T> + Send>>,
}

impl<T: Send> AsyncFuture<T> {
    pub fn new<F>(future: F) -> Self where F: Future<Output = T> + Send + 'static {
        Self { inner: Box::pin(future) }
    }
    pub fn map<U, F>(self, f: F) -> AsyncFuture<U> where F: FnOnce(T) -> U + Send + 'static, U: Send + 'static {
        Self::new(async move { f(self.await) })
    }
    pub fn then<U, F, Fut>(self, f: F) -> AsyncFuture<U> where F: FnOnce(T) -> Fut + Send + 'static, Fut: Future<Output = U> + Send + 'static, U: Send + 'static {
        Self::new(async move { f(self.await).await })
    }
}

impl<T: Send, E: Send> AsyncFuture<Result<T, E>> {
    pub fn ok(self) -> AsyncFuture<Option<T>> { self.map(|r| r.ok()) }
    pub fn err(self) -> AsyncFuture<Option<E>> { self.map(|r| r.err()) }
}

impl<F: Future + Send> Future for AsyncFuture<F::Output> where F::Output: Send {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().inner.as_mut().poll(cx)
    }
}

/// Trait for types that need async cleanup.
pub trait AsyncDrop { fn async_drop(self); }

/// Wrapper providing AsyncDrop for types with cleanup closures.
pub struct AsyncDropper<T> {
    value: T,
    cleanup: Option<Box<dyn FnOnce(T) -> Pin<Box<dyn Future<Output = ()> + Send>>> + Send>>,
}

impl<T: Send> AsyncDropper<T> {
    pub fn new<F, Fut>(value: T, cleanup: F) -> Self where F: FnOnce(T) -> Fut + Send + 'static, Fut: Future<Output = ()> + Send + 'static {
        Self { value, cleanup: Some(Box::new(|v| Box::pin(cleanup(v)))) }
    }
}

impl<T: Send> AsyncDrop for AsyncDropper<T> {
    fn async_drop(mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            let value = std::mem::take(&mut self.value);
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
        let future = AsyncFuture::new(async { 42 });
        let mapped = future.map(|v| v * 2);
        assert_eq!(mapped.await, 84);
    }

    #[tokio::test]
    async fn test_async_future_then() {
        let future = AsyncFuture::new(async { 42 });
        let chained = future.then(|v| async move { v + 8 });
        assert_eq!(chained.await, 50);
    }

    #[tokio::test]
    async fn test_async_future_result() {
        let ok_future: AsyncFuture<Result<i32, &str>> = AsyncFuture::new(async { Ok::<_, &str>(42) });
        assert_eq!(ok_future.clone().ok().await, Some(42));
        assert_eq!(ok_future.err().await, None);
    }

    #[test]
    fn test_async_dropper() {
        static CALLED: AtomicUsize = AtomicUsize::new(0);
        struct TestValue(i32);
        { let dropper = AsyncDropper::new(TestValue(42), |val| async move { CALLED.store(val.0, Ordering::SeqCst); }); dropper.async_drop(); }
        assert_eq!(CALLED.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_collect_vec_size_hint() {
        struct TestIter;
        impl Iterator for TestIter {
            type Item = i32;
            fn next(&mut self) -> Option<Self::Item> { Some(1) }
            fn size_hint(&self) -> (usize, Option<usize>) { (5, Some(5)) }
        }
        impl AsyncIterator for TestIter {
            type Item = i32;
            fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> { Poll::Ready(self.get_mut().next()) }
            fn size_hint(&self) -> (usize, Option<usize>) { Iterator::size_hint(self) }
        }
        let collector = CollectVec::new(TestIter);
        let (min, max) = collector.size_hint();
        assert_eq!(min, 0);
        assert_eq!(max, Some(5));
    }
}
