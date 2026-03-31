//! Async trait helpers for Phenotype ecosystem.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Type alias for async cleanup functions.
type AsyncCleanupFn<T> = Box<dyn FnOnce(T) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// AsyncIterator trait - async version of the standard Iterator.
pub trait AsyncIterator {
    type Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<T: AsyncIterator + Unpin> AsyncIterator for &mut T {
    type Item = T::Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // SAFETY: T: Unpin, so we can project through the reference.
        let inner: Pin<&mut T> = unsafe { Pin::new_unchecked(&mut **self.get_unchecked_mut()) };
        inner.poll_next(cx)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

/// Extension trait for AsyncIterator with utilities.
pub trait AsyncIteratorExt: AsyncIterator {
    fn collect_vec(self) -> CollectVec<Self>
    where
        Self: Sized + Unpin,
    {
        CollectVec::new(self)
    }
}

impl<T: AsyncIterator> AsyncIteratorExt for T {}

/// Collector that accumulates items into a vector.
pub struct CollectVec<I: AsyncIterator> {
    iterator: I,
    items: Vec<I::Item>,
}

impl<I: AsyncIterator> CollectVec<I> {
    pub fn new(iterator: I) -> Self {
        Self {
            iterator,
            items: Vec::new(),
        }
    }

    pub fn into_vec(self) -> Vec<I::Item> {
        self.items
    }
}

// SAFETY: If I: Unpin, then CollectVec<I> is also Unpin because all its fields are Unpin.
impl<I: AsyncIterator + Unpin> Unpin for CollectVec<I> {}

impl<I: AsyncIterator + Unpin> AsyncIterator for CollectVec<I> {
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // SAFETY: I: Unpin allows us to get a mutable reference to the inner iterator.
        let this = unsafe { self.get_unchecked_mut() };
        let iterator = unsafe { Pin::new_unchecked(&mut this.iterator) };
        match iterator.poll_next(cx) {
            Poll::Ready(Some(item)) => {
                this.items.push(item);
                Poll::Pending
            }
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

/// Wrapper for boxed async futures.
pub struct AsyncFuture<T> {
    inner: Pin<Box<dyn Future<Output = T> + Send>>,
}

impl<T: Send + 'static> AsyncFuture<T> {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Self {
            inner: Box::pin(future),
        }
    }

    pub fn map<U: Send + 'static, M>(self, f: M) -> AsyncFuture<U>
    where
        M: FnOnce(T) -> U + Send + 'static,
    {
        let fut = self.inner;
        AsyncFuture::new(async move { f(fut.await) })
    }

    pub fn then<U, G, Fut>(self, g: G) -> AsyncFuture<U>
    where
        G: FnOnce(T) -> Fut + Send + 'static,
        Fut: Future<Output = U> + Send + 'static,
        U: Send + 'static,
    {
        let fut = self.inner;
        AsyncFuture::new(async move { g(fut.await).await })
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

/// Wrapper providing AsyncDrop for types with cleanup closures.
pub struct AsyncDropper<T> {
    value: Option<T>,
    cleanup: Option<AsyncCleanupFn<T>>,
}

impl<T: Send + 'static> AsyncDropper<T> {
    pub fn new<F, Fut>(value: T, cleanup: F) -> Self
    where
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            value: Some(value),
            cleanup: Some(Box::new(move |v| Box::pin(cleanup(v)))),
        }
    }

    pub async fn async_drop(&mut self) {
        if let Some(value) = self.value.take() {
            if let Some(cleanup) = self.cleanup.take() {
                cleanup(value).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_map() {
        assert_eq!(AsyncFuture::new(async { 42 }).map(|v| v * 2).await, 84);
    }

    #[tokio::test]
    async fn test_then() {
        assert_eq!(
            AsyncFuture::new(async { 42 })
                .then(|v| async move { v + 8 })
                .await,
            50
        );
    }

    #[tokio::test]
    async fn test_ok() {
        let f: AsyncFuture<Result<i32, &str>> = AsyncFuture::new(async { Ok::<_, &str>(42) });
        assert_eq!(f.ok().await, Some(42));
    }

    #[tokio::test]
    async fn test_err() {
        let f: AsyncFuture<Result<i32, &str>> = AsyncFuture::new(async { Err::<i32, _>("e") });
        assert_eq!(f.err().await, Some("e"));
    }

    #[tokio::test]
    async fn test_new() {
        assert_eq!(AsyncFuture::new(async { "hi" }).await, "hi");
    }

    #[tokio::test]
    async fn test_async_dropper() {
        static CALLED: AtomicUsize = AtomicUsize::new(0);
        struct TestValue(i32);
        {
            let mut dropper = AsyncDropper::new(TestValue(42), |val| async move {
                CALLED.store(val.0 as usize, Ordering::SeqCst);
            });
            dropper.async_drop().await;
        }
        assert_eq!(CALLED.load(Ordering::SeqCst), 42);
    }
}
