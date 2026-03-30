//! Windowing and chunking iterators.
//!
//! Provides utilities for sliding windows, fixed-size chunks, and overlapping windows
//! over iterators.

use std::collections::VecDeque;

/// A windowed iterator that yields consecutive items in fixed-size windows.
///
/// # Examples
///
/// ```
/// use phenotype_iter::window::WindowedIter;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let mut windowed = WindowedIter::new(data.iter().copied(), 2);
/// assert_eq!(windowed.next(), Some(vec![1, 2]));
/// assert_eq!(windowed.next(), Some(vec![2, 3]));
/// ```
pub struct WindowedIter<I: Iterator> {
    iter: I,
    window_size: usize,
    buffer: VecDeque<I::Item>,
}

impl<I: Iterator> WindowedIter<I>
where
    I::Item: Clone,
{
    /// Creates a new windowed iterator with the specified window size.
    ///
    /// # Panics
    ///
    /// Panics if `window_size` is 0.
    pub fn new(iter: I, window_size: usize) -> Self {
        assert!(window_size > 0, "window size must be greater than 0");
        Self {
            iter,
            window_size,
            buffer: VecDeque::with_capacity(window_size),
        }
    }

    /// Creates a new windowed iterator that yields non-overlapping chunks.
    pub fn chunks(iter: I, chunk_size: usize) -> ChunkedIter<I> {
        assert!(chunk_size > 0, "chunk size must be greater than 0");
        ChunkedIter {
            iter,
            chunk_size,
            buffer: VecDeque::with_capacity(chunk_size),
        }
    }
}

impl<I: Iterator> Iterator for WindowedIter<I>
where
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        // Fill the initial window
        while self.buffer.len() < self.window_size {
            match self.iter.next() {
                Some(item) => self.buffer.push_back(item),
                None => {
                    if self.buffer.is_empty() {
                        return None;
                    }
                    return None;
                }
            }
        }

        // Yield current window
        let window = self.buffer.iter().cloned().collect::<Vec<_>>();

        // Slide: remove first, add next if available
        if let Some(next_item) = self.iter.next() {
            self.buffer.pop_front();
            self.buffer.push_back(next_item);
            Some(window)
        } else {
            None
        }
    }
}

/// A chunked iterator that yields non-overlapping consecutive items.
///
/// # Examples
///
/// ```
/// use phenotype_iter::window::ChunkedIter;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let mut chunked = ChunkedIter::new(data.iter().copied(), 2);
/// assert_eq!(chunked.next(), Some(vec![1, 2]));
/// assert_eq!(chunked.next(), Some(vec![3, 4]));
/// assert_eq!(chunked.next(), Some(vec![5]));
/// ```
pub struct ChunkedIter<I: Iterator> {
    iter: I,
    chunk_size: usize,
    buffer: VecDeque<I::Item>,
}

impl<I: Iterator> ChunkedIter<I>
where
    I::Item: Clone,
{
    /// Creates a new chunked iterator with the specified chunk size.
    ///
    /// # Panics
    ///
    /// Panics if `chunk_size` is 0.
    pub fn new(iter: I, chunk_size: usize) -> Self {
        assert!(chunk_size > 0, "chunk size must be greater than 0");
        Self {
            iter,
            chunk_size,
            buffer: VecDeque::with_capacity(chunk_size),
        }
    }
}

impl<I: Iterator> Iterator for ChunkedIter<I>
where
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();

        for _ in 0..self.chunk_size {
            match self.iter.next() {
                Some(item) => self.buffer.push_back(item),
                None => break,
            }
        }

        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.iter().cloned().collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windowed_iter_basic() {
        let data = vec![1, 2, 3, 4, 5];
        let mut windowed = WindowedIter::new(data.iter().copied(), 2);
        assert_eq!(windowed.next(), Some(vec![1, 2]));
        assert_eq!(windowed.next(), Some(vec![2, 3]));
        assert_eq!(windowed.next(), Some(vec![3, 4]));
        assert_eq!(windowed.next(), Some(vec![4, 5]));
        assert_eq!(windowed.next(), None);
    }

    #[test]
    fn test_windowed_iter_larger_window() {
        let data = vec![1, 2, 3, 4, 5];
        let mut windowed = WindowedIter::new(data.iter().copied(), 3);
        assert_eq!(windowed.next(), Some(vec![1, 2, 3]));
        assert_eq!(windowed.next(), Some(vec![2, 3, 4]));
        assert_eq!(windowed.next(), Some(vec![3, 4, 5]));
        assert_eq!(windowed.next(), None);
    }

    #[test]
    fn test_windowed_iter_window_size_one() {
        let data = vec![1, 2, 3];
        let mut windowed = WindowedIter::new(data.iter().copied(), 1);
        assert_eq!(windowed.next(), None); // Window size 1 requires sliding
    }

    #[test]
    fn test_windowed_iter_window_exceeds_data() {
        let data = vec![1, 2];
        let mut windowed = WindowedIter::new(data.iter().copied(), 5);
        assert_eq!(windowed.next(), None);
    }

    #[test]
    fn test_windowed_iter_empty() {
        let data: Vec<i32> = vec![];
        let mut windowed = WindowedIter::new(data.iter().copied(), 2);
        assert_eq!(windowed.next(), None);
    }

    #[test]
    fn test_chunked_iter_basic() {
        let data = vec![1, 2, 3, 4, 5];
        let mut chunked = ChunkedIter::new(data.iter().copied(), 2);
        assert_eq!(chunked.next(), Some(vec![1, 2]));
        assert_eq!(chunked.next(), Some(vec![3, 4]));
        assert_eq!(chunked.next(), Some(vec![5]));
        assert_eq!(chunked.next(), None);
    }

    #[test]
    fn test_chunked_iter_exact_chunks() {
        let data = vec![1, 2, 3, 4];
        let mut chunked = ChunkedIter::new(data.iter().copied(), 2);
        assert_eq!(chunked.next(), Some(vec![1, 2]));
        assert_eq!(chunked.next(), Some(vec![3, 4]));
        assert_eq!(chunked.next(), None);
    }

    #[test]
    fn test_chunked_iter_single_chunk() {
        let data = vec![1, 2, 3];
        let mut chunked = ChunkedIter::new(data.iter().copied(), 10);
        assert_eq!(chunked.next(), Some(vec![1, 2, 3]));
        assert_eq!(chunked.next(), None);
    }

    #[test]
    fn test_chunked_iter_empty() {
        let data: Vec<i32> = vec![];
        let mut chunked = ChunkedIter::new(data.iter().copied(), 2);
        assert_eq!(chunked.next(), None);
    }

    #[test]
    #[should_panic(expected = "window size must be greater than 0")]
    fn test_windowed_zero_size() {
        WindowedIter::new(vec![1, 2, 3].iter().copied(), 0);
    }

    #[test]
    #[should_panic(expected = "chunk size must be greater than 0")]
    fn test_chunked_zero_size() {
        ChunkedIter::new(vec![1, 2, 3].iter().copied(), 0);
    }
}
