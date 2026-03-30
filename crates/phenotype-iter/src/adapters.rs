//! Iterator adapter implementations for advanced iteration patterns.

use std::collections::VecDeque;

/// An iterator adapter that groups consecutive elements into fixed-size chunks.
///
/// This is a lazy iterator that only materializes chunks when they are consumed.
/// The last chunk may contain fewer elements than the requested chunk size.
///
/// # Example
///
/// ```
/// use phenotype_iter::ChunkIter;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let mut chunks = ChunkIter::new(items.into_iter(), 2);
/// assert_eq!(chunks.next(), Some(vec![1, 2]));
/// assert_eq!(chunks.next(), Some(vec![3, 4]));
/// assert_eq!(chunks.next(), Some(vec![5]));
/// assert_eq!(chunks.next(), None);
/// ```
#[derive(Debug, Clone)]
pub struct ChunkIter<I: Iterator> {
    iter: I,
    size: usize,
}

impl<I: Iterator> ChunkIter<I> {
    /// Creates a new chunk iterator with the specified chunk size.
    ///
    /// # Arguments
    ///
    /// * `iter` - The underlying iterator to chunk
    /// * `size` - The size of each chunk (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
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
///
/// This is a lazy iterator that maintains a sliding window of the specified size,
/// yielding a new window for each element in the underlying iterator.
/// Only cloneable items can be used with window iteration.
///
/// # Example
///
/// ```
/// use phenotype_iter::WindowIter;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let windows: Vec<Vec<i32>> = WindowIter::new(items.into_iter(), 3)
///     .collect();
/// assert_eq!(windows[0], vec![1, 2, 3]);
/// assert_eq!(windows[1], vec![2, 3, 4]);
/// assert_eq!(windows[2], vec![3, 4, 5]);
/// ```
#[derive(Debug, Clone)]
pub struct WindowIter<T: Clone> {
    buffer: VecDeque<T>,
    window_size: usize,
    iter: Box<dyn Iterator<Item = T>>,
    exhausted: bool,
}

impl<T: Clone + 'static> WindowIter<T> {
    /// Creates a new window iterator with the specified window size.
    ///
    /// # Arguments
    ///
    /// * `iter` - The underlying iterator to window
    /// * `size` - The size of each window (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    pub fn new<I: Iterator<Item = T> + 'static>(iter: I, size: usize) -> Self {
        assert!(size > 0, "window size must be greater than 0");
        WindowIter {
            buffer: VecDeque::with_capacity(size),
            window_size: size,
            iter: Box::new(iter),
            exhausted: false,
        }
    }
}

impl<T: Clone> Iterator for WindowIter<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        // Fill initial window
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

        // Create window from buffer
        let window: Vec<T> = self.buffer.iter().cloned().collect();

        // Try to advance buffer for next iteration
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
///
/// This is a lazy iterator that accumulates elements into batches,
/// starting a new batch whenever the predicate function returns true
/// for the current element.
///
/// # Example
///
/// ```
/// use phenotype_iter::BatchIter;
///
/// let items = vec![1, 2, 3, 4, 5, 6];
/// let batches: Vec<Vec<i32>> = BatchIter::new(items.into_iter(), |x| x % 3 == 1)
///     .collect();
/// // Batches are: [1], [2, 3], [4], [5, 6]
/// assert_eq!(batches[0], vec![1]);
/// assert_eq!(batches[1], vec![2, 3]);
/// ```
#[derive(Debug)]
pub struct BatchIter<I: Iterator, F>
where
    F: Fn(&I::Item) -> bool,
{
    iter: Option<I>,
    predicate: F,
    current_batch: Vec<I::Item>,
    exhausted: bool,
}

impl<I: Iterator, F> BatchIter<I, F>
where
    F: Fn(&I::Item) -> bool,
{
    /// Creates a new batch iterator with the given predicate.
    ///
    /// # Arguments
    ///
    /// * `iter` - The underlying iterator to batch
    /// * `predicate` - A function that returns `true` to start a new batch
    pub fn new(iter: I, predicate: F) -> Self {
        BatchIter {
            iter: Some(iter),
            predicate,
            current_batch: Vec::new(),
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
        if self.exhausted {
            return None;
        }

        let mut iter = match self.iter.take() {
            Some(it) => it,
            None => {
                return if self.current_batch.is_empty() {
                    None
                } else {
                    Some(std::mem::take(&mut self.current_batch))
                };
            }
        };

        let mut completed_batch = None;

        for item in iter.by_ref() {
            if (self.predicate)(&item) && !self.current_batch.is_empty() {
                // Start new batch with this item
                completed_batch = Some(std::mem::take(&mut self.current_batch));
                self.current_batch.push(item);
                self.iter = Some(iter);
                return completed_batch;
            }
            self.current_batch.push(item);
        }

        // Iterator exhausted
        self.exhausted = true;

        if completed_batch.is_some() {
            self.iter = None;
            completed_batch
        } else if self.current_batch.is_empty() {
            self.iter = None;
            None
        } else {
            self.iter = None;
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
    fn chunk_iter_single_item() {
        let items = vec![1, 2, 3];
        let chunks: Vec<Vec<i32>> = ChunkIter::new(items.into_iter(), 1).collect();
        assert_eq!(chunks, vec![vec![1], vec![2], vec![3]]);
    }

    #[test]
    fn chunk_iter_empty() {
        let items: Vec<i32> = vec![];
        let chunks: Vec<Vec<i32>> = ChunkIter::new(items.into_iter(), 2).collect();
        assert_eq!(chunks, vec![]);
    }

    #[test]
    fn window_iter_basic() {
        let items = vec![1, 2, 3, 4, 5];
        let windows: Vec<Vec<i32>> = WindowIter::new(items.into_iter(), 3).collect();
        assert_eq!(
            windows,
            vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]
        );
    }

    #[test]
    fn window_iter_window_larger_than_sequence() {
        let items = vec![1, 2];
        let windows: Vec<Vec<i32>> = WindowIter::new(items.into_iter(), 3).collect();
        assert_eq!(windows, vec![vec![1, 2]]);
    }

    #[test]
    fn window_iter_single_window() {
        let items = vec![1, 2, 3];
        let windows: Vec<Vec<i32>> = WindowIter::new(items.into_iter(), 3).collect();
        assert_eq!(windows, vec![vec![1, 2, 3]]);
    }

    #[test]
    fn window_iter_empty() {
        let items: Vec<i32> = vec![];
        let windows: Vec<Vec<i32>> = WindowIter::new(items.into_iter(), 2).collect();
        assert_eq!(windows, vec![]);
    }

    #[test]
    fn batch_iter_basic() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let batches: Vec<Vec<i32>> = BatchIter::new(items.into_iter(), |x| x % 3 == 1).collect();
        assert_eq!(batches, vec![vec![1], vec![2, 3], vec![4], vec![5, 6]]);
    }

    #[test]
    fn batch_iter_no_batches() {
        let items = vec![1, 2, 3];
        let batches: Vec<Vec<i32>> =
            BatchIter::new(items.into_iter(), |_x| false).collect();
        assert_eq!(batches, vec![vec![1, 2, 3]]);
    }

    #[test]
    fn batch_iter_all_batches() {
        let items = vec![1, 2, 3];
        let batches: Vec<Vec<i32>> =
            BatchIter::new(items.into_iter(), |_x| true).collect();
        assert_eq!(batches, vec![vec![1], vec![2], vec![3]]);
    }

    #[test]
    fn batch_iter_empty() {
        let items: Vec<i32> = vec![];
        let batches: Vec<Vec<i32>> =
            BatchIter::new(items.into_iter(), |_x| false).collect();
        assert_eq!(batches, vec![]);
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
        let _iter: WindowIter<i32> = WindowIter::new(items.into_iter(), 0);
    }
}
