//! FlatMap iterator adapter (map then flatten).

/// Iterator adapter that maps each item and then flattens the results.
///
/// # Example
/// ```
/// use phenotype_iter::IterExt;
///
/// let items = vec![1, 2, 3];
/// let result: Vec<i32> = items
///     .into_iter()
///     .flat_map_custom(|x| vec![x, x * 2])
///     .collect();
/// assert_eq!(result, vec![1, 2, 2, 4, 3, 6]);
/// ```
pub struct FlatMapCustom<I, F, U>
where
    I: Iterator,
    F: Fn(I::Item) -> U,
    U: IntoIterator,
{
    iter: I,
    map_fn: F,
    current: Option<U::IntoIter>,
}

impl<I, F, U> FlatMapCustom<I, F, U>
where
    I: Iterator,
    F: Fn(I::Item) -> U,
    U: IntoIterator,
{
    /// Create a new flatmap iterator.
    pub fn new(iter: I, map_fn: F) -> Self {
        FlatMapCustom {
            iter,
            map_fn,
            current: None,
        }
    }
}

impl<I, F, U> Iterator for FlatMapCustom<I, F, U>
where
    I: Iterator,
    F: Fn(I::Item) -> U,
    U: IntoIterator,
{
    type Item = U::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to get the next item from the current inner iterator
            if let Some(ref mut inner) = self.current {
                if let Some(item) = inner.next() {
                    return Some(item);
                }
            }

            // Current iterator exhausted, get the next outer item
            match self.iter.next() {
                Some(item) => {
                    let inner_iter = (self.map_fn)(item).into_iter();
                    self.current = Some(inner_iter);
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
    fn flat_map_basic() {
        let items = vec![1, 2, 3];
        let result: Vec<i32> = items
            .into_iter()
            .flat_map(|x| vec![x, x * 2])
            .collect();
        assert_eq!(result, vec![1, 2, 2, 4, 3, 6]);
    }

    #[test]
    fn flat_map_string_split() {
        let items = vec!["a,b", "c,d,e"];
        let result: Vec<&str> = items
            .into_iter()
            .flat_map(|s| s.split(',').collect::<Vec<_>>())
            .collect();
        assert_eq!(result, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn flat_map_empty_inner() {
        let items = vec![1, 2, 3];
        let result: Vec<i32> = items
            .into_iter()
            .flat_map(|x| {
                if x == 2 {
                    vec![]
                } else {
                    vec![x]
                }
            })
            .collect();
        assert_eq!(result, vec![1, 3]);
    }

    #[test]
    fn flat_map_empty_outer() {
        let items: Vec<i32> = vec![];
        let result: Vec<i32> = items
            .into_iter()
            .flat_map(|x| vec![x, x * 2])
            .collect();
        assert!(result.is_empty());
    }

    #[test]
    fn flat_map_range() {
        let items = vec![1, 2, 3];
        let result: Vec<i32> = items
            .into_iter()
            .flat_map(|x| (0..x).collect::<Vec<_>>())
            .collect();
        assert_eq!(result, vec![0, 0, 1, 0, 1, 2]);
    }
}
