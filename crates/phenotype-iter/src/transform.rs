//! Composable transform pipelines for iterators.
//!
//! Provides a builder pattern for chaining map, filter, and fold operations
//! while maintaining type safety and zero-cost abstractions.

use std::marker::PhantomData;

/// A transform pipeline builder that allows composing map and filter operations.
///
/// # Examples
///
/// ```
/// use phenotype_iter::transform::Pipeline;
///
/// let result: Vec<i32> = Pipeline::new(vec![1, 2, 3, 4, 5].into_iter())
///     .map(|x| x * 2)
///     .filter(|x| x > &5)
///     .collect();
/// assert_eq!(result, vec![6, 8, 10]);
/// ```
pub struct Pipeline<I: Iterator, T> {
    iter: I,
    _phantom: PhantomData<T>,
}

impl<I: Iterator> Pipeline<I, I::Item> {
    /// Creates a new pipeline from an iterator.
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            _phantom: PhantomData,
        }
    }
}

impl<I: Iterator> Pipeline<I, I::Item> {
    /// Applies a map transformation to the pipeline.
    ///
    /// # Examples
    ///
    /// ```
    /// use phenotype_iter::transform::Pipeline;
    ///
    /// let result: Vec<i32> = Pipeline::new(vec![1, 2, 3].into_iter())
    ///     .map(|x| x * 2)
    ///     .collect();
    /// assert_eq!(result, vec![2, 4, 6]);
    /// ```
    pub fn map<F, U>(self, f: F) -> Pipeline<std::iter::Map<I, F>, U>
    where
        F: FnMut(I::Item) -> U,
    {
        Pipeline {
            iter: self.iter.map(f),
            _phantom: PhantomData,
        }
    }

    /// Applies a filter transformation to the pipeline.
    ///
    /// # Examples
    ///
    /// ```
    /// use phenotype_iter::transform::Pipeline;
    ///
    /// let result: Vec<i32> = Pipeline::new(vec![1, 2, 3, 4, 5].into_iter())
    ///     .filter(|x| x % 2 == 0)
    ///     .collect();
    /// assert_eq!(result, vec![2, 4]);
    /// ```
    pub fn filter<F>(self, f: F) -> Pipeline<std::iter::Filter<I, F>, I::Item>
    where
        F: FnMut(&I::Item) -> bool,
    {
        Pipeline {
            iter: self.iter.filter(f),
            _phantom: PhantomData,
        }
    }
}

impl<I: Iterator, T> Pipeline<I, T>
where
    I::Item: Into<T>,
{
    /// Applies a filter-map transformation.
    pub fn filter_map<F, U>(self, f: F) -> Pipeline<std::iter::FilterMap<I, F>, U>
    where
        F: FnMut(I::Item) -> Option<U>,
    {
        Pipeline {
            iter: self.iter.filter_map(f),
            _phantom: PhantomData,
        }
    }
}

impl<I: Iterator, T> Iterator for Pipeline<I, T>
where
    I::Item: Into<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| x.into())
    }
}

/// Convenience trait for chaining operations on any iterator.
pub trait TransformExt: Iterator + Sized {
    /// Creates a new pipeline from this iterator.
    fn pipe(self) -> Pipeline<Self, Self::Item> {
        Pipeline::new(self)
    }

    /// Applies multiple map operations efficiently.
    fn map_chain<F1, F2, U1, U2>(self, f1: F1, f2: F2) -> std::iter::Map<std::iter::Map<Self, F1>, F2>
    where
        F1: FnMut(Self::Item) -> U1,
        F2: FnMut(U1) -> U2,
    {
        self.map(f1).map(f2)
    }

    /// Applies a filter operation in the pipeline.
    fn filter_gate<F>(self, f: F) -> std::iter::Filter<Self, F>
    where
        F: FnMut(&Self::Item) -> bool,
    {
        self.filter(f)
    }
}

impl<I: Iterator> TransformExt for I {}

/// A fold operation builder that accumulates values with type transformations.
///
/// # Examples
///
/// ```
/// use phenotype_iter::transform::Fold;
///
/// let result = Fold::new(0)
///     .with_iter(vec![1, 2, 3].into_iter())
///     .fold(|acc, x| acc + x);
/// assert_eq!(result, 6);
/// ```
pub struct Fold<T, I: Iterator> {
    accumulator: T,
    iter: I,
}

impl<T, I: Iterator> Fold<T, I> {
    /// Creates a new fold operation with an initial accumulator.
    pub fn new(accumulator: T) -> FoldBuilder<T> {
        FoldBuilder { accumulator }
    }

    /// Applies the fold operation with a given function.
    pub fn fold<F>(self, f: F) -> T
    where
        F: Fn(T, I::Item) -> T,
    {
        self.iter.fold(self.accumulator, f)
    }

    /// Applies the fold operation and returns both the result and final value.
    pub fn fold_with_count<F>(self, f: F) -> (T, usize)
    where
        F: Fn(T, I::Item) -> T,
    {
        let mut count = 0;
        let result = self.iter.fold(self.accumulator, |acc, item| {
            count += 1;
            f(acc, item)
        });
        (result, count)
    }
}

/// Builder for fold operations.
pub struct FoldBuilder<T> {
    accumulator: T,
}

impl<T> FoldBuilder<T> {
    /// Attaches an iterator to the fold builder.
    pub fn with_iter<I: Iterator>(self, iter: I) -> Fold<T, I> {
        Fold {
            accumulator: self.accumulator,
            iter,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_map() {
        let result: Vec<i32> = Pipeline::new(vec![1, 2, 3].into_iter())
            .map(|x| x * 2)
            .collect();
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_pipeline_filter() {
        let result: Vec<i32> = Pipeline::new(vec![1, 2, 3, 4, 5].into_iter())
            .filter(|x| x % 2 == 0)
            .collect();
        assert_eq!(result, vec![2, 4]);
    }

    #[test]
    fn test_pipeline_chained() {
        let result: Vec<i32> = Pipeline::new(vec![1, 2, 3, 4, 5].into_iter())
            .map(|x| x * 2)
            .filter(|x| x > &5)
            .collect();
        assert_eq!(result, vec![6, 8, 10]);
    }

    #[test]
    fn test_pipeline_empty() {
        let result: Vec<i32> = Pipeline::new(vec![].into_iter())
            .map(|x| x * 2)
            .collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_pipeline_filter_all_out() {
        let result: Vec<i32> = Pipeline::new(vec![1, 2, 3].into_iter())
            .filter(|x| x > &10)
            .collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_pipeline_complex_chain() {
        let result: Vec<String> = Pipeline::new(vec![1, 2, 3, 4, 5, 6].into_iter())
            .filter(|x| x % 2 == 0)
            .map(|x| format!("num_{}", x))
            .collect();
        assert_eq!(result, vec!["num_2", "num_4", "num_6"]);
    }

    #[test]
    fn test_transform_ext_pipe() {
        let result: Vec<i32> = vec![1, 2, 3]
            .into_iter()
            .pipe()
            .map(|x| x * 2)
            .collect();
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_transform_ext_map_chain() {
        let result: Vec<i32> = vec![1, 2, 3]
            .into_iter()
            .map_chain(|x| x * 2, |x| x + 1)
            .collect();
        assert_eq!(result, vec![3, 5, 7]);
    }

    #[test]
    fn test_transform_ext_filter_gate() {
        let result: Vec<i32> = vec![1, 2, 3, 4, 5]
            .into_iter()
            .filter_gate(|x| x > &2)
            .collect();
        assert_eq!(result, vec![3, 4, 5]);
    }

    #[test]
    fn test_fold_basic() {
        let result = Fold::new(0)
            .with_iter(vec![1, 2, 3].into_iter())
            .fold(|acc, x| acc + x);
        assert_eq!(result, 6);
    }

    #[test]
    fn test_fold_string_accumulation() {
        let result = Fold::new(String::new())
            .with_iter(vec!["a", "b", "c"].into_iter())
            .fold(|mut acc, x| {
                acc.push_str(x);
                acc
            });
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_fold_with_count() {
        let (result, count) = Fold::new(0)
            .with_iter(vec![1, 2, 3, 4, 5].into_iter())
            .fold_with_count(|acc, x| acc + x);
        assert_eq!(result, 15);
        assert_eq!(count, 5);
    }

    #[test]
    fn test_fold_empty_iter() {
        let result = Fold::new(42)
            .with_iter(vec![].into_iter())
            .fold(|acc, _x| acc);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_fold_with_count_empty() {
        let (result, count) = Fold::new(100)
            .with_iter(vec![].into_iter())
            .fold_with_count(|acc, _x| acc);
        assert_eq!(result, 100);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_fold_string_join() {
        let result = Fold::new(Vec::new())
            .with_iter(vec![1, 2, 3].into_iter())
            .fold(|mut acc, x| {
                acc.push(x);
                acc
            });
        assert_eq!(result, vec![1, 2, 3]);
    }
}
