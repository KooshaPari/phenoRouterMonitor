//! Sliding window iterator adapter.

/// Iterator adapter that yields a sliding window of items.
///
/// # Example
/// ```
/// use phenotype_iter::IterExt;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let windows: Vec<Vec<i32>> = items.into_iter().window(2).collect();
/// assert_eq!(windows, vec![vec![1, 2], vec![2, 3], vec![3, 4], vec![4, 5]]);
/// ```
pub struct Window<I: Iterator> {
    iter: I,
    window: Vec<I::Item>,
    size: usize,
}

impl<I: Iterator> Window<I> {
    /// Create a new sliding window iterator.
    pub fn new(iter: I, size: usize) -> Self {
        if size == 0 {
            panic!("window size must be at least 1");
        }
        Window {
            iter,
            window: Vec::with_capacity(size),
            size,
        }
    }
}

impl<I> Iterator for Window<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        // Fill the window initially
        while self.window.len() < self.size {
            match self.iter.next() {
                Some(item) => self.window.push(item),
                None => return None,
            }
        }

        let result = self.window.clone();

        // Slide the window: remove first, add new
        self.window.remove(0);
        if let Some(item) = self.iter.next() {
            self.window.push(item);
            Some(result)
        } else {
            // Return the last full window if we've exhausted the iterator
            if self.window.is_empty() {
                None
            } else {
                Some(result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IterExt;

    #[test]
    fn window_basic() {
        let items = vec![1, 2, 3, 4, 5];
        let windows: Vec<Vec<i32>> = items.into_iter().window(2).collect();
        assert_eq!(
            windows,
            vec![vec![1, 2], vec![2, 3], vec![3, 4], vec![4, 5]]
        );
    }

    #[test]
    fn window_larger() {
        let items = vec![1, 2, 3, 4, 5];
        let windows: Vec<Vec<i32>> = items.into_iter().window(3).collect();
        assert_eq!(
            windows,
            vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]
        );
    }

    #[test]
    fn window_equal_to_length() {
        let items = vec![1, 2, 3];
        let windows: Vec<Vec<i32>> = items.into_iter().window(3).collect();
        assert_eq!(windows, vec![vec![1, 2, 3]]);
    }

    #[test]
    fn window_larger_than_length() {
        let items = vec![1, 2];
        let windows: Vec<Vec<i32>> = items.into_iter().window(3).collect();
        assert!(windows.is_empty());
    }

    #[test]
    fn window_single() {
        let items = vec![1, 2, 3, 4];
        let windows: Vec<Vec<i32>> = items.into_iter().window(1).collect();
        assert_eq!(
            windows,
            vec![vec![1], vec![2], vec![3], vec![4]]
        );
    }

    #[test]
    fn window_empty() {
        let items: Vec<i32> = vec![];
        let windows: Vec<Vec<i32>> = items.into_iter().window(2).collect();
        assert!(windows.is_empty());
    }
}
