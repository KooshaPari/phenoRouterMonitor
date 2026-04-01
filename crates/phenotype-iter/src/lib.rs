//! # phenotype-iter: Advanced Iterator Utilities
//!
//! Provides efficient, composable iterator adapters for common patterns:
//! - **Windowing**: Sliding windows over sequences
//! - **Chunking**: Fixed-size non-overlapping partitions
//! - **Batching**: Predicate-based grouping with pending item tracking
//!
//! All implementations use lazy evaluation for memory efficiency and support
//! generic iterators via trait extensions.
//!
//! # Examples
//!
//! ```ignore
//! use phenotype_iter::Window;
//!
//! let data = vec![1, 2, 3, 4, 5];
//! let windows: Vec<_> = data.iter().window(3).collect();
//! // windows = [[1,2,3], [2,3,4], [3,4,5]]
//! ```

use std::collections::VecDeque;

/// Trait for creating sliding windows over an iterator.
///
/// Each window contains up to `size` consecutive elements. Windows overlap,
/// advancing by one element per iteration.
pub trait Window: Iterator + Sized
where
    Self::Item: Clone,
{
    /// Create a sliding window iterator with the given window size.
    ///
    /// # Panics
    /// Panics if `size` is 0.
    fn window(self, size: usize) -> WindowIter<Self> {
        assert!(size > 0, "window size must be greater than 0");
        WindowIter {
            iter: self,
            buffer: VecDeque::with_capacity(size),
            window_size: size,
            exhausted: false,
        }
    }
}

impl<I: Iterator> Window for I where I::Item: Clone {}

/// An iterator adapter that yields sliding windows.
pub struct WindowIter<I: Iterator>
where
    I::Item: Clone,
{
    iter: I,
    buffer: VecDeque<I::Item>,
    window_size: usize,
    exhausted: bool,
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

        // Fill buffer to window_size on first call or when rebuilding
        while self.buffer.len() < self.window_size {
            match self.iter.next() {
                Some(item) => self.buffer.push_back(item),
                None => {
                    self.exhausted = true;
                    // Return partial window if we have any items
                    if self.buffer.is_empty() {
                        return None;
                    }
                    break;
                }
            }
        }

        if self.buffer.is_empty() {
            return None;
        }

        let window: Vec<_> = self.buffer.iter().cloned().collect();

        // Advance window by removing front and trying to add next item
        self.buffer.pop_front();
        
        // Try to get next item from iterator to add to back
        if let Some(item) = self.iter.next() {
            self.buffer.push_back(item);
        } else {
            self.exhausted = true;
        }

        Some(window)
    }
}

/// Trait for chunking an iterator into fixed-size non-overlapping groups.
pub trait Chunk: Iterator + Sized
where
    Self::Item: Clone,
{
    /// Create a chunking iterator with the given chunk size.
    ///
    /// # Panics
    /// Panics if `size` is 0.
    fn chunk(self, size: usize) -> ChunkIter<Self> {
        assert!(size > 0, "chunk size must be greater than 0");
        ChunkIter {
            iter: self,
            buffer: Vec::with_capacity(size),
            size,
        }
    }
}

impl<I: Iterator> Chunk for I where I::Item: Clone {}

/// An iterator adapter that yields fixed-size chunks.
pub struct ChunkIter<I: Iterator>
where
    I::Item: Clone,
{
    iter: I,
    buffer: Vec<I::Item>,
    size: usize,
}

impl<I: Iterator> Iterator for ChunkIter<I>
where
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();

        for _ in 0..self.size {
            match self.iter.next() {
                Some(item) => self.buffer.push(item),
                None => break,
            }
        }

        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.clone())
        }
    }
}

impl<I: Iterator> ExactSizeIterator for ChunkIter<I> where I::Item: Clone + ExactSizeIterator {}

/// Trait for batching an iterator based on predicates.
///
/// Batches are created by grouping consecutive elements that satisfy a predicate.
/// When the predicate returns false, a new batch begins.
pub trait Batch: Iterator + Sized {
    /// Create a batching iterator using the given predicate.
    ///
    /// Items continue in the current batch while `predicate` returns true.
    /// When it returns false, the current batch ends and a new one begins.
    fn batch<F>(self, predicate: F) -> BatchIter<Self, F>
    where
        F: Fn(&Self::Item) -> bool,
    {
        BatchIter {
            iter: self,
            predicate,
            buffer: Vec::new(),
            pending: None,
            exhausted: false,
        }
    }
}

impl<I: Iterator> Batch for I {}

/// An iterator adapter that yields predicate-based batches.
pub struct BatchIter<I: Iterator, F: Fn(&I::Item) -> bool> {
    iter: I,
    predicate: F,
    buffer: Vec<I::Item>,
    pending: Option<I::Item>,
    exhausted: bool,
}

impl<I: Iterator, F: Fn(&I::Item) -> bool> Iterator for BatchIter<I, F> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        // Return buffered items if any (from previous incomplete batch)
        if !self.buffer.is_empty() {
            return Some(std::mem::take(&mut self.buffer));
        }

        // If exhausted, we're done
        if self.exhausted {
            return None;
        }

        // Handle pending item from previous iteration
        if let Some(item) = self.pending.take() {
            if (self.predicate)(&item) {
                self.buffer.push(item);
            } else {
                // Item doesn't match predicate, start new batch with it
                self.buffer.push(item);
                self.exhausted = true;
            }
        }

        // If we had a non-matching pending item and accumulated it, return it as batch
        if !self.buffer.is_empty() && self.exhausted {
            return Some(std::mem::take(&mut self.buffer));
        }

        // Fill batch while predicate returns true
        while let Some(item) = self.iter.next() {
            if (self.predicate)(&item) {
                self.buffer.push(item);
            } else {
                self.pending = Some(item);
                break;
            }
        }

        if self.buffer.is_empty() {
            self.exhausted = true;
            None
        } else {
            Some(std::mem::take(&mut self.buffer))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_basic() {
        let data = vec![1, 2, 3, 4, 5];
        let windows: Vec<_> = data.into_iter().window(2).collect();
        assert_eq!(windows.len(), 4);
        assert_eq!(windows[0], vec![1, 2]);
        assert_eq!(windows[1], vec![2, 3]);
        assert_eq!(windows[3], vec![4, 5]);
    }

    #[test]
    fn test_window_size_one() {
        let data = vec![1, 2, 3];
        let windows: Vec<_> = data.into_iter().window(1).collect();
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0], vec![1]);
        assert_eq!(windows[2], vec![3]);
    }

    #[test]
    fn test_window_larger_than_input() {
        let data = vec![1, 2];
        let windows: Vec<_> = data.into_iter().window(5).collect();
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0], vec![1, 2]);
    }

    #[test]
    fn test_chunk_basic() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let chunks: Vec<_> = data.into_iter().chunk(2).collect();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], vec![1, 2]);
        assert_eq!(chunks[1], vec![3, 4]);
        assert_eq!(chunks[2], vec![5, 6]);
    }

    #[test]
    fn test_chunk_uneven() {
        let data = vec![1, 2, 3, 4, 5];
        let chunks: Vec<_> = data.into_iter().chunk(2).collect();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[2], vec![5]);
    }

    #[test]
    fn test_chunk_size_one() {
        let data = vec![1, 2, 3];
        let chunks: Vec<_> = data.into_iter().chunk(1).collect();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], vec![1]);
    }

    #[test]
    fn test_batch_basic() {
        let data = vec![1, 2, 3, 4, 5];
        let batches: Vec<_> = data.into_iter().batch(|&x| x < 4).collect();
        assert!(batches.len() >= 1);
        assert_eq!(batches[0], vec![1, 2, 3]);
    }

    #[test]
    fn test_batch_all_match() {
        let data = vec![1, 2, 3];
        let batches: Vec<_> = data.into_iter().batch(|&x| x > 0).collect();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0], vec![1, 2, 3]);
    }

    #[test]
    fn test_batch_none_match() {
        let data = vec![1, 2, 3];
        let batches: Vec<_> = data.into_iter().batch(|&x| x > 10).collect();
        assert_eq!(batches.len(), 0);
    }

    #[test]
    fn test_batch_alternating() {
        let data = vec![1, 3, 2, 4];
        let batches: Vec<_> = data.into_iter().batch(|&x| x % 2 == 1).collect();
        assert!(batches.len() >= 1);
    }

    #[test]
    fn test_window_empty() {
        let data: Vec<i32> = vec![];
        let windows: Vec<_> = data.into_iter().window(3).collect();
        assert_eq!(windows.len(), 0);
    }

    #[test]
    fn test_chunk_empty() {
        let data: Vec<i32> = vec![];
        let chunks: Vec<_> = data.into_iter().chunk(3).collect();
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_batch_empty() {
        let data: Vec<i32> = vec![];
        let batches: Vec<_> = data.into_iter().batch(|_| true).collect();
        assert_eq!(batches.len(), 0);
    }

    #[test]
    fn test_window_order_preserved() {
        let data = vec!['a', 'b', 'c', 'd'];
        let windows: Vec<_> = data.into_iter().window(3).collect();
        assert_eq!(windows[0], vec!['a', 'b', 'c']);
        assert_eq!(windows[1], vec!['b', 'c', 'd']);
    }

    #[test]
    fn test_chunk_order_preserved() {
        let data = vec!['a', 'b', 'c', 'd', 'e'];
        let chunks: Vec<_> = data.into_iter().chunk(2).collect();
        assert_eq!(chunks[0], vec!['a', 'b']);
        assert_eq!(chunks[1], vec!['c', 'd']);
    }

    #[test]
    fn test_composition_window_then_chunk() {
        let data = vec![1, 2, 3, 4];
        let result: Vec<_> = data.into_iter().window(2).flatten().chunk(2).collect();
        assert!(result.len() > 0);
    }
}
