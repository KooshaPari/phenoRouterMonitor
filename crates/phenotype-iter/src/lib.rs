#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

//! # phenotype-iter
//!
//! Advanced iteration utilities for lazy, memory-efficient batch processing.
//!
//! This crate provides composable iterator adapters that enable efficient processing
//! of large sequences without allocating intermediate collections.
//!
//! ## Features
//!
//! - **Chunk Iterator**: Group elements into fixed-size chunks
//! - **Window Iterator**: Sliding window over consecutive elements
//! - **Batch Iterator**: Accumulate elements until a condition is met
//!
//! All iterators are lazy and memory-efficient, only materializing data when needed.
//!
//! ## Examples
//!
//! ```
//! use phenotype_iter::Chunk;
//!
//! let items = vec![1, 2, 3, 4, 5, 6, 7];
//! let chunks: Vec<Vec<i32>> = items
//!     .into_iter()
//!     .chunk(3)
//!     .collect();
//! assert_eq!(chunks, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]]);
//! ```

use std::collections::VecDeque;
use thiserror::Error;

/// Errors produced by iteration operations.
#[derive(Debug, Error)]
#[error("{0}")]
pub enum Error {
    /// Invalid configuration or state.
    Invalid(String),
}

/// Result type for iteration operations.
pub type Result<T> = std::result::Result<T, Error>;

// ============================================================================
// Traits
// ============================================================================

/// Trait for chunking an iterator into fixed-size groups.
///
/// This trait extends any iterator to provide efficient grouping
/// of consecutive elements without pre-allocating the full collection.
pub trait Chunk: Iterator + Sized {
    /// Creates an iterator that groups elements into chunks of the given size.
    ///
    /// The last chunk may contain fewer elements than the requested size.
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    fn chunk(self, size: usize) -> ChunkIter<Self> {
        assert!(size > 0, "chunk size must be greater than 0");
        ChunkIter::new(self, size)
    }
}

impl<I: Iterator> Chunk for I {}

/// Trait for creating sliding windows over an iterator.
///
/// This trait extends any iterator to provide efficient sliding
/// window operations without materializing the entire collection.
pub trait Windowed: Iterator + Sized {
    /// Creates an iterator that yields sliding windows of the given size.
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    fn window(self, size: usize) -> WindowIter<Self>
    where
        Self::Item: Clone,
    {
        assert!(size > 0, "window size must be greater than 0");
        WindowIter::new(self, size)
    }
}

impl<I: Iterator> Windowed for I {}

/// Trait for batching an iterator based on a predicate.
///
/// This trait extends any iterator to provide efficient accumulation
/// of elements into batches determined by a condition function.
pub trait Batch: Iterator + Sized {
    /// Creates an iterator that batches elements using the given predicate.
    ///
    /// A new batch is started whenever the predicate returns true for
    /// the current element.
    fn batch<F>(self, predicate: F) -> BatchIter<Self, F>
    where
        F: Fn(&Self::Item) -> bool,
    {
        BatchIter::new(self, predicate)
    }
}

impl<I: Iterator> Batch for I {}

// ============================================================================
// Iterator Adapters
// ============================================================================

/// An iterator adapter that groups consecutive elements into fixed-size chunks.
#[derive(Debug, Clone)]
pub struct ChunkIter<I: Iterator> {
    iter: I,
    size: usize,
}

impl<I: Iterator> ChunkIter<I> {
    /// Creates a new chunk iterator with the specified chunk size.
    pub fn new(iter: I, size: usize) -> Self {
        assert!(size > 0, "chunk size must be greater than 0");
        ChunkIter { iter, size }
    }
}

impl<I: Iterator> Iterator for ChunkIter<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.size);
        for _ in 0..self.size {
            match self.iter.next() {
                Some(item) => chunk.push(item),
                None => break,
            }
        }

        if chunk.is_empty() {
            None
        } else {
            Some(chunk)
        }
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for ChunkIter<I> {
    fn len(&self) -> usize {
        (self.iter.len() + self.size - 1) / self.size
    }
}

/// An iterator adapter that creates sliding windows over consecutive elements.
#[derive(Debug)]
pub struct WindowIter<I: Iterator> {
    buffer: VecDeque<I::Item>,
    window_size: usize,
    iter: I,
    exhausted: bool,
}

impl<I: Iterator> WindowIter<I>
where
    I::Item: Clone,
{
    /// Creates a new window iterator with the specified window size.
    pub fn new(iter: I, size: usize) -> Self {
        assert!(size > 0, "window size must be greater than 0");
        WindowIter {
            buffer: VecDeque::with_capacity(size),
            window_size: size,
            iter,
            exhausted: false,
        }
    }
}

impl<I: Iterator> Iterator for WindowIter<I>
where
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        while self.buffer.len() < self.window_size {
            match self.iter.next() {
                Some(item) => self.buffer.push_back(item),
                None => {
                    if self.buffer.is_empty() {
                        self.exhausted = true;
                        return None;
                    }
                    self.exhausted = true;
                    return Some(self.buffer.iter().cloned().collect());
                }
            }
        }

        let window: Vec<I::Item> = self.buffer.iter().cloned().collect();

        match self.iter.next() {
            Some(item) => {
                self.buffer.pop_front();
                self.buffer.push_back(item);
                Some(window)
            }
            None => {
                self.exhausted = true;
                Some(window)
            }
        }
    }
}

/// An iterator adapter that batches elements based on a predicate function.
#[derive(Debug)]
pub struct BatchIter<I: Iterator, F>
where
    F: Fn(&I::Item) -> bool,
{
    iter: Option<I>,
    predicate: F,
    current_batch: Vec<I::Item>,
    pending_item: Option<I::Item>,
    exhausted: bool,
}

impl<I: Iterator, F> BatchIter<I, F>
where
    F: Fn(&I::Item) -> bool,
{
    /// Creates a new batch iterator with the given predicate.
    pub fn new(iter: I, predicate: F) -> Self {
        BatchIter {
            iter: Some(iter),
            predicate,
            current_batch: Vec::new(),
            pending_item: None,
            exhausted: false,
        }
    }
}

impl<I: Iterator, F> Iterator for BatchIter<I, F>
where
    F: Fn(&I::Item) -> bool,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted && self.current_batch.is_empty() && self.pending_item.is_none() {
            return None;
        }

        let mut iter = match self.iter.take() {
            Some(it) => it,
            None => {
                if let Some(item) = self.pending_item.take() {
                    self.current_batch.push(item);
                }
                self.exhausted = true;
                return if self.current_batch.is_empty() {
                    None
                } else {
                    Some(std::mem::take(&mut self.current_batch))
                };
            }
        };

        // Handle pending item from previous iteration
        if let Some(item) = self.pending_item.take() {
            self.current_batch.push(item);
        }

        for item in iter.by_ref() {
            if (self.predicate)(&item) && !self.current_batch.is_empty() {
                // Save the triggering item for the next batch
                self.pending_item = Some(item);
                self.iter = Some(iter);
                return Some(std::mem::take(&mut self.current_batch));
            }
            self.current_batch.push(item);
        }

        // Iterator exhausted
        self.exhausted = true;
        self.iter = None;

        if self.current_batch.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.current_batch))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_iter_basic() {
        let items = vec![1, 2, 3, 4, 5, 6, 7];
        let chunks: Vec<Vec<i32>> = ChunkIter::new(items.into_iter(), 3).collect();
        assert_eq!(chunks, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]]);
    }

    #[test]
    fn chunk_iter_exact_size() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let chunks: Vec<Vec<i32>> = ChunkIter::new(items.into_iter(), 3).collect();
        assert_eq!(chunks, vec![vec![1, 2, 3], vec![4, 5, 6]]);
    }

    #[test]
    fn window_iter_basic() {
        let items = vec![1, 2, 3, 4, 5];
        let windows: Vec<Vec<i32>> = items.into_iter().window(3).collect();
        assert_eq!(
            windows,
            vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]
        );
    }

    #[test]
    fn batch_iter_basic() {
        // When predicate is true, a new batch starts with that item
        let items = vec![1, 2, 3, 4, 5, 6];
        let batches: Vec<Vec<i32>> = BatchIter::new(items.into_iter(), |x: &i32| x % 3 == 1).collect();
        // Predicate true for 1 and 4, so batches are: [1,2,3] then [4,5,6]
        assert_eq!(batches, vec![vec![1, 2, 3], vec![4, 5, 6]]);
    }

    #[test]
    #[should_panic(expected = "chunk size must be greater than 0")]
    fn chunk_iter_zero_size_panics() {
        let items = vec![1, 2, 3];
        let _iter: ChunkIter<_> = ChunkIter::new(items.into_iter(), 0);
    }

    #[test]
    #[should_panic(expected = "window size must be greater than 0")]
    fn window_iter_zero_size_panics() {
        let items = vec![1, 2, 3];
        let _iter: Vec<Vec<i32>> = items.into_iter().window(0).collect();
    }
}
