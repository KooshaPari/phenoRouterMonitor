//! # Phenotype Iter
//!
//! Iterator extensions: batching, dedup by key, and collection helpers.

/// Extension trait for iterators.
pub trait IterExt: Iterator + Sized {
    /// Collect items into fixed-size batches.
    ///
    /// The last batch may be smaller than `size`.
    fn batched(self, size: usize) -> Batched<Self> {
        Batched { iter: self, size }
    }

    /// Deduplicate consecutive items by a key function.
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
}

impl<I: Iterator> IterExt for I {}

/// Iterator adapter that yields batches (Vec) of items.
pub struct Batched<I> {
    iter: I,
    size: usize,
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

/// Iterator adapter that deduplicates consecutive items by key.
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
}
