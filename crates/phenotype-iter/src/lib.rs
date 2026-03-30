//! Iterator extensions for common patterns.

use std::collections::HashSet;
use std::hash::Hash;

/// Extension trait for iterators providing common utility methods.
pub trait IterExt: Iterator + Sized {
    /// Collect iterator into a Vec, returning error if any item is an error.
    fn try_collect_vec<T, E>(self) -> Result<Vec<T>, E>
    where
        Self: Iterator<Item = Result<T, E>>,
    {
        let mut vec = Vec::new();
        for item in self {
            vec.push(item?);
        }
        Ok(vec)
    }

    /// Return unique elements in iteration order (first occurrence).
    fn unique_by<T, K, F>(self, key_fn: F) -> UniqueBy<Self, F, K>
    where
        Self: Iterator<Item = T>,
        F: Fn(&T) -> K,
        K: Eq + Hash,
    {
        UniqueBy {
            iter: self,
            key_fn,
            seen: HashSet::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Chunk iterator into chunks of fixed size.
    fn chunk_by_size(self, chunk_size: usize) -> ChunkBySize<Self>
    where
        Self: Iterator,
    {
        assert!(chunk_size > 0, "chunk_size must be > 0");
        ChunkBySize {
            iter: self,
            chunk_size,
            buffer: Vec::with_capacity(chunk_size),
        }
    }

    /// Interleave elements from two iterators.
    fn interleave<I>(self, other: I) -> Interleave<Self, I::IntoIter>
    where
        Self: Iterator,
        I: IntoIterator<Item = Self::Item>,
    {
        Interleave {
            iter1: self,
            iter2: other.into_iter(),
            turn: true,
        }
    }
}

impl<I: Iterator> IterExt for I {}

/// Iterator adapter for unique_by.
pub struct UniqueBy<I, F, K> {
    iter: I,
    key_fn: F,
    seen: HashSet<u64>,
    _phantom: std::marker::PhantomData<K>,
}

impl<I, T, K, F> Iterator for UniqueBy<I, F, K>
where
    I: Iterator<Item = T>,
    F: Fn(&T) -> K,
    K: Eq + Hash,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        while let Some(item) = self.iter.next() {
            let key = (self.key_fn)(&item);
            let key_hash = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::Hasher;
                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                hasher.finish()
            };
            if self.seen.insert(key_hash) {
                return Some(item);
            }
        }
        None
    }
}

/// Iterator adapter for chunk_by_size.
pub struct ChunkBySize<I: Iterator> {
    iter: I,
    chunk_size: usize,
    buffer: Vec<I::Item>,
}

impl<I: Iterator> Iterator for ChunkBySize<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Vec<I::Item>> {
        self.buffer.clear();
        for _ in 0..self.chunk_size {
            match self.iter.next() {
                Some(item) => self.buffer.push(item),
                None => break,
            }
        }
        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.drain(..).collect())
        }
    }
}

/// Iterator adapter for interleave.
pub struct Interleave<I1, I2> {
    iter1: I1,
    iter2: I2,
    turn: bool,
}

impl<I1, I2> Iterator for Interleave<I1, I2>
where
    I1: Iterator,
    I2: Iterator<Item = I1::Item>,
{
    type Item = I1::Item;

    fn next(&mut self) -> Option<I1::Item> {
        if self.turn {
            self.turn = false;
            self.iter1.next().or_else(|| {
                self.turn = true;
                self.iter2.next()
            })
        } else {
            self.turn = true;
            self.iter2.next().or_else(|| {
                self.turn = false;
                self.iter1.next()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_collect_vec_success() {
        let data: Vec<Result<i32, String>> = vec![Ok(1), Ok(2), Ok(3)];
        let result = data.into_iter().try_collect_vec();
        assert_eq!(result, Ok(vec![1, 2, 3]));
    }

    #[test]
    fn test_try_collect_vec_error() {
        let data: Vec<Result<i32, String>> = vec![Ok(1), Err("oops".to_string()), Ok(3)];
        let result: Result<Vec<_>, String> = data.into_iter().try_collect_vec();
        assert!(result.is_err());
    }

    #[test]
    fn test_chunk_by_size() {
        let data = vec![1, 2, 3, 4, 5, 6, 7];
        let chunks: Vec<_> = data.into_iter().chunk_by_size(3).collect();
        assert_eq!(chunks, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]]);
    }

    #[test]
    fn test_interleave() {
        let iter1 = vec![1, 2, 3].into_iter();
        let iter2 = vec![10, 20, 30].into_iter();
        let result: Vec<_> = iter1.interleave(iter2).collect();
        assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
    }

    #[test]
    fn test_interleave_uneven() {
        let iter1 = vec![1, 2].into_iter();
        let iter2 = vec![10, 20, 30, 40].into_iter();
        let result: Vec<_> = iter1.interleave(iter2).collect();
        // Results should contain all items from both iterators
        assert_eq!(result.len(), 6);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&10));
        assert!(result.contains(&20));
        assert!(result.contains(&30));
        assert!(result.contains(&40));
    }
}
