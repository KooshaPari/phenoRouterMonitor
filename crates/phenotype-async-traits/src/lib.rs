//! Async trait helpers for Phenotype ecosystem.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// AsyncIterator trait.
pub trait AsyncIterator {
    type Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
    fn size_hint(&self) -> (usize, Option<usize>) { (0, None) }
}

pub trait AsyncIteratorExt: AsyncIterator {
    fn collect_vec(self) -> CollectVec<Self> where Self: Sized { CollectVec::new(self) }
}
impl<T: AsyncIterator> AsyncIteratorExt for T {}

pub struct CollectVec<I: AsyncIterator> { iterator: I, items: Vec<I::Item> }
impl<I: AsyncIterator> CollectVec<I> { fn new(iterator: I) -> Self { Self { iterator, items: Vec::new() } } }
impl<I: AsyncIterator + Unpin> Unpin for CollectVec<I> {}

impl<I: AsyncIterator + Unpin> AsyncIterator for CollectVec<I> {
    type Item = I::Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
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

pub struct AsyncFuture<T> { inner: Pin<Box<dyn Future<Output = T> + Send>> }

impl<T: Send + 'static> AsyncFuture<T> {
    pub fn new<F: Future<Output = T> + Send + 'static>(future: F) -> Self {
        Self { inner: Box::pin(future) }
    }
    pub fn map<U: Send + 'static, M: FnOnce(T) -> U + Send + 'static>(self, f: M) -> AsyncFuture<U> {
        let fut = self.inner;
        AsyncFuture::new(async move { f(fut.await) })
    }
    pub fn then<U: Send + 'static, G: FnOnce(T) -> Fut + Send + 'static, Fut: Future<Output = U> + Send + 'static>(self, f: G) -> AsyncFuture<U> {
        let fut = self.inner;
        AsyncFuture::new(async move { f(fut.await).await })
    }
}

impl<T: Send + 'static, E: Send + 'static> AsyncFuture<Result<T, E>> {
    pub fn ok(self) -> AsyncFuture<Option<T>> { self.map(|r| r.ok()) }
    pub fn err(self) -> AsyncFuture<Option<E>> { self.map(|r| r.err()) }
}

impl<T: Send + 'static> Future for AsyncFuture<T> {
    type Output = T;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.as_mut().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test] async fn test_map() {
        assert_eq!(AsyncFuture::new(async { 42 }).map(|v| v * 2).await, 84);
    }
    #[tokio::test] async fn test_then() {
        assert_eq!(AsyncFuture::new(async { 42 }).then(|v| async move { v + 8 }).await, 50);
    }
    #[tokio::test] async fn test_ok() {
        let f: AsyncFuture<Result<i32, &str>> = AsyncFuture::new(async { Ok::<_, &str>(42) });
        assert_eq!(f.ok().await, Some(42));
    }
    #[tokio::test] async fn test_err() {
        let f: AsyncFuture<Result<i32, &str>> = AsyncFuture::new(async { Err::<i32, _>("e") });
        assert_eq!(f.err().await, Some("e"));
    }
    #[tokio::test] async fn test_new() {
        assert_eq!(AsyncFuture::new(async { "hi" }).await, "hi");
    }
}
