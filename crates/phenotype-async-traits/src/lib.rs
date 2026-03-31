//! Async trait helpers for Phenotype ecosystem.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// AsyncIterator trait - async version of the standard Iterator.
pub trait AsyncIterator {
    type Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub trait AsyncIteratorExt: AsyncIterator {
    fn collect_vec(self) -> CollectVec<Self>
    where
        Self: Sized,
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
    fn new(iterator: I) -> Self {
        Self {
            iterator,
            items: Vec::new(),
        }
    }
}

impl<I: AsyncIterator + Unpin> AsyncIterator for CollectVec<I>
where
    I::Item: Unpin,
{
    type Item = I::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.items.is_empty() {
            return Poll::Ready(Some(self.items.remove(0)));
        }
        Pin::new(&mut self.get_mut().iterator).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let upper = self.iterator.size_hint().1.map(|s| s + self.items.len());
        (self.items.len(), upper)
    }
}

impl<I: AsyncIterator> CollectVec<I> {
    pub fn into_vec(self) -> Vec<I::Item> {
        self.items
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

    pub fn map<U: Send + 'static>(self, f: impl FnOnce(T) -> U + Send + 'static) -> AsyncFuture<U> {
        let inner = self.inner;
        AsyncFuture {
            inner: Box::pin(async move { f(inner.await) }),
        }
    }
}

impl<T: Send + 'static> Future for AsyncFuture<T> {
    type Output = T;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.as_mut().poll(cx)
    }
}

/// Trait for types that need async cleanup.
pub trait AsyncDrop {
    fn async_drop(self);
}

pub struct AsyncDropper<T> {
    value: Option<T>,
    cleanup: Option<Box<dyn FnOnce(T)>>,
}

impl<T: Send + 'static> AsyncDropper<T> {
    pub fn new<F>(value: T, cleanup: F) -> Self
    where
        F: FnOnce(T) + Send + 'static,
    {
        Self {
            value: Some(value),
            cleanup: Some(Box::new(cleanup)),
        }
    }
}

impl<T: Send + 'static> AsyncDrop for AsyncDropper<T> {
    fn async_drop(mut self) {
        if let (Some(value), Some(cleanup)) = (self.value.take(), self.cleanup.take()) {
            cleanup(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_async_future_map() {
        let future: AsyncFuture<u32> = AsyncFuture::new(async { 42u32 });
        let mapped: AsyncFuture<u32> = future.map(|v| v * 2);
        let result: u32 = mapped.await;
        assert_eq!(result, 84);
    }

    #[test]
    fn test_async_dropper() {
        static CALLED: AtomicUsize = AtomicUsize::new(0);
        struct TestValue(i32);
        {
            let dropper = AsyncDropper::new(TestValue(42), |val| {
                CALLED.store(val.0 as usize, Ordering::SeqCst);
            });
            dropper.async_drop();
        }
        assert_eq!(CALLED.load(Ordering::SeqCst), 42);
    }

    #[tokio::test]
    async fn test_async_future_new() {
        let future: AsyncFuture<&'static str> = AsyncFuture::new(async { "hello" });
        let result: &str = future.await;
        assert_eq!(result, "hello");
    }
}
