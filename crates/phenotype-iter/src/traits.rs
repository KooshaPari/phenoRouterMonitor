//! Iterator adapter traits for advanced iteration patterns.

/// Trait for chunking an iterator into fixed-size groups.
///
/// This trait extends any iterator to provide efficient grouping
/// of consecutive elements without pre-allocating the full collection.
///
/// # Example
///
/// ```
/// use phenotype_iter::Chunk;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let result: Vec<Vec<i32>> = items
///     .into_iter()
///     .chunk(2)
///     .collect();
/// assert_eq!(result, vec![vec![1, 2], vec![3, 4], vec![5]]);
/// ```
pub trait Chunk: Iterator + Sized {
    /// Creates an iterator that groups elements into chunks of the given size.
    ///
    /// The last chunk may contain fewer elements than the requested size.
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    fn chunk(self, size: usize) -> crate::adapters::ChunkIter<Self> {
        assert!(size > 0, "chunk size must be greater than 0");
        crate::adapters::ChunkIter::new(self, size)
    }
}

impl<I: Iterator> Chunk for I {}

/// Trait for creating sliding windows over an iterator.
///
/// This trait extends any iterator to provide efficient sliding
/// window operations without materializing the entire collection.
///
/// # Example
///
/// ```
/// use phenotype_iter::Windowed;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let result: Vec<Vec<i32>> = items
///     .into_iter()
///     .window(3)
///     .collect();
/// assert_eq!(result, vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]);
/// ```
pub trait Windowed: Iterator + Sized {
    /// Creates an iterator that yields sliding windows of the given size.
    ///
    /// Each window contains consecutive elements, with each new window
    /// shifting by exactly one position.
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    fn window(self, size: usize) -> crate::adapters::WindowIter<Self::Item>
    where
        Self::Item: Clone,
    {
        assert!(size > 0, "window size must be greater than 0");
        crate::adapters::WindowIter::new(self, size)
    }
}

impl<I: Iterator> Windowed for I {}

/// Trait for batching an iterator based on a predicate.
///
/// This trait extends any iterator to provide efficient accumulation
/// of elements into batches determined by a condition function.
///
/// # Example
///
/// ```
/// use phenotype_iter::Batch;
///
/// let items = vec![1, 2, 3, 4, 5, 6];
/// let result: Vec<Vec<i32>> = items
///     .into_iter()
///     .batch(|item| item % 3 == 0)
///     .collect();
/// assert_eq!(result, vec![vec![1, 2, 3], vec![4, 5, 6]]);
/// ```
pub trait Batch: Iterator + Sized {
    /// Creates an iterator that batches elements using the given predicate.
    ///
    /// A new batch is started whenever the predicate returns true for
    /// the current element.
    ///
    /// # Arguments
    ///
    /// * `predicate` - A function that returns `true` to start a new batch
    ///
    /// # Example
    ///
    /// ```
    /// use phenotype_iter::Batch;
    ///
    /// let items = vec![1, 2, 3, 4, 5, 6];
    /// let batches: Vec<Vec<i32>> = items
    ///     .into_iter()
    ///     .batch(|x| x % 3 == 1)
    ///     .collect();
    /// // Batches when item % 3 == 1: [1], [2, 3], [4], [5, 6]
    /// assert_eq!(batches[0], vec![1]);
    /// assert_eq!(batches[1], vec![2, 3]);
    /// ```
    fn batch<F>(self, predicate: F) -> crate::adapters::BatchIter<Self, F>
    where
        F: Fn(&Self::Item) -> bool,
    {
        crate::adapters::BatchIter::new(self, predicate)
    }
}

impl<I: Iterator> Batch for I {}
