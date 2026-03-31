//! Async trait helpers for Phenotype ecosystem.
//!
//! This crate provides async versions of common std traits and utilities
//! for working with async/await in Rust.

use std::future::Future;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::task::{Context, Poll};

/// AsyncIterator trait - async version of the standard Iterator.
pub trait AsyncIterator {
    type Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
    fn size_hint(&self) -> (usize, Option<usize>) { (0, None) }
}

impl<T: AsyncIterator + Unpin> AsyncIterator for &mut T {
    type Item = T::Item;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut *self).poll_next(cx)
    }
    fn size_hint(&self) -> (usize, Option<usize>) { (**self).size_hint() }
}

/// AsyncIterator extension trait with utilities.
pub trait AsyncIteratorExt: AsyncIterator {
    fn collect_vec(self) -> CollectVec<Self> where Self: Sized { CollectVec::new(self) }
}

impl<T: AsyncIterator> AsyncIteratorExt for T {}

/// Collector that accumulates items into a vector.
pub struct CollectVec<I: AsyncIterator> { iterator: I, items: Vec<I::Item>, _pinned: PhantomPinned }

impl<I: AsyncIterator> CollectVec<I> {
    fn new(iterator: I) -> Self { Self { iterator, items: Vec::new(), _pinned: PhantomPinned } }
}

impl<I: AsyncIterator + Unpin> AsyncIterator for CollectVec<I> {
    type Item = I::Item;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;
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

impl<I: AsyncIterator> CollectVec<I> {
    pub fn into_vec(self) -> Vec<I::Item> { self.items }
}

/// Wrapper for boxed async futures.
#[derive(Clone)]
pub struct AsyncFuture<T> {
    inner: Pin<Box<dyn Future<Output = T> + Send>>,
}

impl<T> AsyncFuture<T> {
    pub fn new<F>(future: F) -> Self 
    where
        F: Future<Output = T> + Send + 'static,
        T: Send,
    {
        Self { inner: Box::pin(future) }
    }
}

impl<T: Send + 'static, F: Future<Output = T> + Send + 'static> AsyncFuture<T> {
    pub fn map<U, M>(self, f: M) -> AsyncFuture<U>
    where
        M: FnOnce(T) -> U + Send + 'static,
        U: Send + 'static,
    {
        let fut = self.inner;
        AsyncFuture::new(async move { f(fut.await) })
    }
    
    pub fn then<U, G, Fut>(self, f: G) -> AsyncFuture<U>
    where
        G: FnOnce(T) -> Fut + Send + 'static,
        Fut: Future<Output = U> + Send + 'static,
        U: Send + 'static,
    {
        let fut = self.inner;
        AsyncFuture::new(async move { f(fut.await).await })
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

impl<F: Future + Send> Future for AsyncFuture<F::Output> {
    type Output = F::Output;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.as_mut().poll(cx)
    }
}

/// Trait for types that need async cleanup.
pub trait AsyncDrop {
    fn async_drop(self);
}

/// Wrapper providing AsyncDrop for types with cleanup closures.
pub struct AsyncDropper<T> {
    value: Option<T>,
    cleanup: Option<Box<dyn FnOnce(T) -> Pin<Box<dyn Future<Output = ()> + Send>>> + Send>>,
}

impl<T: Send + 'static> AsyncDropper<T> {
    pub fn new<F, Fut>(value: T, cleanup: F) -> Self 
    where 
        F: FnOnce(T) -> Fut + Send + 'static, 
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self { value: Some(value), cleanup: Some(Box::new(|v| Box::pin(cleanup(v)))) }
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
    async fn test_async_future_result_ok() {
        let ok_future: AsyncFuture<Result<i32, &str>> = AsyncFuture::new(async { Ok::<_, &str>(42) });
        let ok_fut = ok_future.ok();
        let err_fut = ok_future.err();
        assert_eq!(ok_fut.await, Some(42));
        assert_eq!(err_fut.await, None);
    }

    #[test]
    fn test_async_dropper() {
        static CALLED: AtomicUsize = AtomicUsize::new(0);
        struct TestValue(i32);
        { 
            let dropper = AsyncDropper::new(TestValue(42), |val| async move { 
                CALLED.store(val.0 as usize, Ordering::SeqCst); 
            }); 
            dropper.async_drop(); 
        }
        assert_eq!(CALLED.load(Ordering::SeqCst), 42);
    }

    #[tokio::test]
    async fn test_async_future_result_err() {
        let err_future: AsyncFuture<Result<i32, &str>> = AsyncFuture::new(async { Err::<i32, _>("error") });
        let ok_fut = err_future.ok();
        let err_fut = err_future.err();
        assert_eq!(ok_fut.await, None);
        assert_eq!(err_fut.await, Some("error"));
    }

    #[tokio::test]
    async fn test_async_future_new() {
        let future = AsyncFuture::new(async { "hello" });
        assert_eq!(future.await, "hello");
    }
}
