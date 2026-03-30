//! # Phenotype Iter
//!
//! Iterator extensions: batching, windowing, deduplication, and flatmapping.
//!
//! This crate provides a collection of zero-copy iterator adapters for common patterns:
//! - **Batching**: Group items into fixed-size chunks
//! - **Windowing**: Sliding window over items (requires Clone)
//! - **Deduplication**: Remove duplicate items using custom equality or hashing
//! - **FlatMap**: Map then flatten nested iterators

pub mod batch;
pub mod dedup;
pub mod flatten_map;
pub mod window;

pub use batch::Batched;
pub use dedup::{DedupCustom, DedupHashed};
pub use flatten_map::FlatMapCustom;
pub use window::Window;

/// Extension trait for iterators.
///
/// Provides convenient methods for common iterator patterns.
pub trait IterExt: Iterator + Sized {
    /// Collect items into fixed-size batches.
    ///
    /// The last batch may be smaller than `size`.
    ///
    /// # Panics
    /// Panics if `size` is zero.
    ///
    /// # Example
    /// ```
    /// use phenotype_iter::IterExt;
    ///
    /// let batches: Vec<Vec<i32>> = (1..=5).batched(2).collect();
    /// assert_eq!(batches, vec![vec![1, 2], vec![3, 4], vec![5]]);
    /// ```
    fn batched(self, size: usize) -> Batched<Self> {
        Batched::new(self, size)
    }

    /// Create a sliding window over items.
    ///
    /// Requires `Clone` for the item type. Returns windows of the specified size,
    /// sliding one item at a time.
    ///
    /// # Panics
    /// Panics if `size` is zero.
    ///
    /// # Example
    /// ```
    /// use phenotype_iter::IterExt;
    ///
    /// let windows: Vec<Vec<i32>> = vec![1, 2, 3, 4]
    ///     .into_iter()
    ///     .window(2)
    ///     .collect();
    /// assert_eq!(windows, vec![vec![1, 2], vec![2, 3], vec![3, 4]]);
    /// ```
    fn window(self, size: usize) -> Window<Self>
    where
        Self::Item: Clone,
    {
        Window::new(self, size)
    }

    /// Deduplicate consecutive items by a key function.
    ///
    /// Only removes consecutive duplicates, not all duplicates.
    /// For global deduplication, use `dedup_custom` or `dedup_hashed`.
    ///
    /// # Example
    /// ```
    /// use phenotype_iter::IterExt;
    ///
    /// let items = vec![1, 1, 2, 3, 3, 3, 4];
    /// let result: Vec<i32> = items.into_iter()
    ///     .dedup_by_key(|x| *x)
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 3, 4]);
    /// ```
    fn dedup_by_key<K, F>(self, key_fn: F) -> DedupByKey<Self, K, F>
    where
        K: PartialEq,
        F: Fn(&Self::Item) -> K,
    {
        DedupByKey {
            iter: self,
            key_fn,
            last_key: None,
        }
    }

    /// Deduplicate all items using a custom equality function.
    ///
    /// Stores all seen items in memory, so use with caution on large datasets.
    /// For types that implement `Hash + Eq`, prefer `dedup_hashed` for better performance.
    ///
    /// # Example
    /// ```
    /// use phenotype_iter::IterExt;
    ///
    /// let items = vec![1, 2, 2, 1, 3];
    /// let result: Vec<i32> = items.into_iter()
    ///     .dedup_custom(|a, b| a == b)
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
    fn dedup_custom<F>(self, eq_fn: F) -> DedupCustom<Self, F>
    where
        Self::Item: Clone,
        F: Fn(&Self::Item, &Self::Item) -> bool,
    {
        DedupCustom::new(self, eq_fn)
    }

    /// Deduplicate all items for types that implement `Hash + Eq`.
    ///
    /// More efficient than `dedup_custom` for hashable types.
    ///
    /// # Example
    /// ```
    /// use phenotype_iter::IterExt;
    ///
    /// let items = vec![1, 2, 2, 1, 3];
    /// let result: Vec<i32> = items.into_iter().dedup_hashed().collect();
    /// assert_eq!(result, vec![1, 2, 3]);
    /// ```
    fn dedup_hashed(self) -> DedupHashed<Self>
    where
        Self::Item: std::hash::Hash + Eq + Clone,
    {
        DedupHashed::new(self)
    }

    /// Map each item and flatten the results.
    ///
    /// The mapping function should return an `IntoIterator`, which is flattened
    /// into the output stream.
    ///
    /// # Example
    /// ```
    /// use phenotype_iter::IterExt;
    ///
    /// let items = vec![1, 2, 3];
    /// let result: Vec<i32> = items.into_iter()
    ///     .flat_map_custom(|x| vec![x, x * 2])
    ///     .collect();
    /// assert_eq!(result, vec![1, 2, 2, 4, 3, 6]);
    /// ```
    fn flat_map_custom<F, U>(self, map_fn: F) -> FlatMapCustom<Self, F, U>
    where
        F: Fn(Self::Item) -> U,
        U: IntoIterator,
    {
        FlatMapCustom::new(self, map_fn)
    }
}

impl<I: Iterator> IterExt for I {}

/// Iterator adapter that deduplicates consecutive items by key.
///
/// Only removes consecutive duplicates. For global deduplication,
/// use `dedup_custom` or `dedup_hashed`.
pub struct DedupByKey<I: Iterator, K, F> {
    iter: I,
    key_fn: F,
    last_key: Option<K>,
}

impl<I, K, F> Iterator for DedupByKey<I, K, F>
where
    I: Iterator,
    K: PartialEq,
    F: Fn(&I::Item) -> K,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.iter.next()?;
            let key = (self.key_fn)(&item);
            if self.last_key.as_ref() == Some(&key) {
                continue;
            }
            self.last_key = Some(key);
            return Some(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_by_key_consecutive() {
        let items = vec![1, 1, 2, 3, 3, 3, 4, 1];
        let result: Vec<i32> = items.into_iter().dedup_by_key(|x| *x).collect();
        assert_eq!(result, vec![1, 2, 3, 4, 1]); // only consecutive dedup
    }

    #[test]
    fn dedup_by_key_custom() {
        let items = vec!["apple", "ant", "banana", "berry"];
        let result: Vec<&str> = items
            .into_iter()
            .dedup_by_key(|s| s.chars().next().unwrap())
            .collect();
        assert_eq!(result, vec!["apple", "banana"]);
    }

    #[test]
    fn integration_batch_then_dedup() {
        let items = vec![1, 1, 2, 2, 3, 3];
        let batches: Vec<Vec<i32>> = items
            .into_iter()
            .dedup_by_key(|x| *x)
            .batched(2)
            .collect();
        assert_eq!(batches, vec![vec![1, 2], vec![3]]);
    }

    #[test]
    fn integration_flat_map_then_dedup() {
        let items = vec![1, 2];
        let result: Vec<i32> = items
            .into_iter()
            .flat_map(|x| vec![x, x])
            .dedup_hashed()
            .collect();
        assert_eq!(result, vec![1, 2]);
    }
}
