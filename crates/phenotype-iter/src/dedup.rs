//! Deduplication iterator with custom equality.

use std::collections::HashSet;
use std::hash::Hash;

/// Iterator adapter that deduplicates items using a custom equality function.
///
/// This maintains all seen items in memory, so use `dedup_by_key` for
/// consecutive deduplication without memory overhead.
///
/// # Example
/// ```
/// use phenotype_iter::IterExt;
///
/// let items = vec![1, 2, 2, 1, 3];
/// let result: Vec<i32> = items.into_iter().dedup_custom(|a, b| a == b).collect();
/// assert_eq!(result, vec![1, 2, 3]);
/// ```
pub struct DedupCustom<I, F>
where
    I: Iterator,
    F: Fn(&I::Item, &I::Item) -> bool,
{
    iter: I,
    eq_fn: F,
    seen: Vec<I::Item>,
}

impl<I, F> DedupCustom<I, F>
where
    I: Iterator,
    I::Item: Clone,
    F: Fn(&I::Item, &I::Item) -> bool,
{
    /// Create a new dedup iterator with custom equality.
    pub fn new(iter: I, eq_fn: F) -> Self {
        DedupCustom {
            iter,
            eq_fn,
            seen: Vec::new(),
        }
    }
}

impl<I, F> Iterator for DedupCustom<I, F>
where
    I: Iterator,
    I::Item: Clone,
    F: Fn(&I::Item, &I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(item) => {
                    // Check if item is already in seen using the equality function
                    let is_new = !self.seen.iter().any(|seen_item| (self.eq_fn)(seen_item, &item));
                    if is_new {
                        self.seen.push(item.clone());
                        return Some(item);
                    }
                    // Item is duplicate, continue to next
                }
                None => return None,
            }
        }
    }
}

/// Iterator adapter for deduplicating items that implement Hash + Eq.
///
/// More efficient than `dedup_custom` for types that support hashing.
///
/// # Example
/// ```
/// use phenotype_iter::IterExt;
///
/// let items = vec![1, 2, 2, 1, 3];
/// let result: Vec<i32> = items.into_iter().dedup_hashed().collect();
/// assert_eq!(result, vec![1, 2, 3]);
/// ```
pub struct DedupHashed<I>
where
    I: Iterator,
    I::Item: Hash + Eq + Clone,
{
    iter: I,
    seen: HashSet<I::Item>,
}

impl<I> DedupHashed<I>
where
    I: Iterator,
    I::Item: Hash + Eq + Clone,
{
    /// Create a new dedup iterator for hashable types.
    pub fn new(iter: I) -> Self {
        DedupHashed {
            iter,
            seen: HashSet::new(),
        }
    }
}

impl<I> Iterator for DedupHashed<I>
where
    I: Iterator,
    I::Item: Hash + Eq + Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(item) => {
                    if self.seen.insert(item.clone()) {
                        return Some(item);
                    }
                }
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IterExt;

    #[test]
    fn dedup_custom_basic() {
        let items = vec![1, 2, 2, 1, 3];
        let result: Vec<i32> = items
            .into_iter()
            .dedup_custom(|a, b| a == b)
            .collect();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn dedup_custom_strings() {
        let items = vec!["apple", "apricot", "apple", "banana"];
        let result: Vec<&str> = items
            .into_iter()
            .dedup_custom(|a, b| a == b)
            .collect();
        assert_eq!(result, vec!["apple", "apricot", "banana"]);
    }

    #[test]
    fn dedup_custom_case_insensitive() {
        let items = vec!["Apple", "apple", "APPLE", "Banana"];
        let result: Vec<&str> = items
            .into_iter()
            .dedup_custom(|a, b| a.to_lowercase() == b.to_lowercase())
            .collect();
        assert_eq!(result, vec!["Apple", "Banana"]);
    }

    #[test]
    fn dedup_custom_empty() {
        let items: Vec<i32> = vec![];
        let result: Vec<i32> = items
            .into_iter()
            .dedup_custom(|a, b| a == b)
            .collect();
        assert!(result.is_empty());
    }

    #[test]
    fn dedup_hashed_basic() {
        let items = vec![1, 2, 2, 1, 3];
        let result: Vec<i32> = items.into_iter().dedup_hashed().collect();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn dedup_hashed_strings() {
        let items = vec!["apple", "banana", "apple", "cherry"];
        let result: Vec<&str> = items.into_iter().dedup_hashed().collect();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"apple"));
        assert!(result.contains(&"banana"));
        assert!(result.contains(&"cherry"));
    }

    #[test]
    fn dedup_hashed_order_preserved() {
        let items = vec![1, 2, 3, 2, 4, 1, 5];
        let result: Vec<i32> = items.into_iter().dedup_hashed().collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }
}
