//! Batch/chunk iterator adapter.

/// Iterator adapter that yields batches (Vec) of items.
///
/// # Example
/// ```
/// use phenotype_iter::IterExt;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let batches: Vec<Vec<i32>> = items.into_iter().batched(2).collect();
/// assert_eq!(batches, vec![vec![1, 2], vec![3, 4], vec![5]]);
/// ```
pub struct Batched<I> {
    iter: I,
    size: usize,
}

impl<I: Iterator> Batched<I> {
    /// Create a new batched iterator.
    pub fn new(iter: I, size: usize) -> Self {
        if size == 0 {
            panic!("batch size must be at least 1");
        }
        Batched { iter, size }
    }
}

impl<I: Iterator> Iterator for Batched<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut batch = Vec::with_capacity(self.size);
        for _ in 0..self.size {
            match self.iter.next() {
                Some(item) => batch.push(item),
                None => break,
            }
        }
        if batch.is_empty() {
            None
        } else {
            Some(batch)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IterExt;

    #[test]
    fn batched_even() {
        let batches: Vec<Vec<i32>> = (1..=6).batched(2).collect();
        assert_eq!(batches, vec![vec![1, 2], vec![3, 4], vec![5, 6]]);
    }

    #[test]
    fn batched_remainder() {
        let batches: Vec<Vec<i32>> = (1..=7).batched(3).collect();
        assert_eq!(batches, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]]);
    }

    #[test]
    fn batched_empty() {
        let batches: Vec<Vec<i32>> = std::iter::empty::<i32>().batched(5).collect();
        assert!(batches.is_empty());
    }

    #[test]
    fn batched_single_item() {
        let batches: Vec<Vec<i32>> = (1..=1).batched(1).collect();
        assert_eq!(batches, vec![vec![1]]);
    }

    #[test]
    fn batched_batch_size_one() {
        let batches: Vec<Vec<i32>> = (1..=3).batched(1).collect();
        assert_eq!(batches, vec![vec![1], vec![2], vec![3]]);
    }

    #[test]
    fn batched_large_batch() {
        let batches: Vec<Vec<i32>> = (1..=5).batched(10).collect();
        assert_eq!(batches, vec![vec![1, 2, 3, 4, 5]]);
    }
}
