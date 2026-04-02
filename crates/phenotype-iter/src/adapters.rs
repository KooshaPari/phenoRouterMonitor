//! Iterator adapters module.

use std::collections::VecDeque;

/// An iterator adapter that groups consecutive elements into fixed-size chunks.
#[derive(Debug, Clone)]
pub struct ChunkIter<I: Iterator> {
    iter: I,
    size: usize,
}

impl<I: Iterator> ChunkIter<I> {
    /// Creates a new chunk iterator with the specified chunk size.
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
        self.iter.len().div_ceil(self.size)
    }
}

/// An iterator adapter that creates sliding windows over consecutive elements.
#[derive(Debug)]
pub struct WindowIter<I: Iterator> {
    buffer: VecDeque<I::Item>,
    window_size: usize,
    iter: I,
    exhausted: bool,
}

impl<I: Iterator> WindowIter<I>
where
    I::Item: Clone,
{
    /// Creates a new window iterator with the specified window size.
    pub fn new(iter: I, size: usize) -> Self {
        assert!(size > 0, "window size must be greater than 0");
        WindowIter {
            buffer: VecDeque::with_capacity(size),
            window_size: size,
            iter,
            exhausted: false,
        }
    }
}

impl<I: Iterator> Iterator for WindowIter<I>
where
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        // If we're exhausted and no current batch, we're done
        if self.exhausted {
            return None;
        }
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

        let window: Self::Item = self.buffer.iter().cloned().collect();

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
#[derive(Debug)]
pub struct BatchIter<I: Iterator, F>
where
    F: Fn(&I::Item) -> bool,
{
    iter: Option<I>,
    predicate: F,
    current_batch: Vec<I::Item>,
}

impl<I: Iterator, F> BatchIter<I, F>
where
    F: Fn(&I::Item) -> bool,
{
    /// Creates a new batch iterator with the given predicate.
    pub fn new(iter: I, predicate: F) -> Self {
        BatchIter {
            iter: Some(iter),
            predicate,
            current_batch: Vec::new(),
        }
    }
}

impl<I: Iterator, F> Iterator for BatchIter<I, F>
where
    F: Fn(&I::Item) -> bool,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Get next item from iterator
            let item = match self.iter.as_mut().and_then(|i| i.next()) {
                Some(item) => item,
                None => {
                    // Iterator exhausted - return final batch if any
                    if !self.current_batch.is_empty() {
                        return Some(std::mem::take(&mut self.current_batch));
                    }
                    return None;
                }
            };

            if (self.predicate)(&item) {
                // Predicate true: yield current batch if non-empty, start new
                if !self.current_batch.is_empty() {
                    let batch = std::mem::take(&mut self.current_batch);
                    self.current_batch.push(item);
                    return Some(batch);
                }
                // Start new batch with this item
                self.current_batch.push(item);
            } else {
                // False: accumulate into current batch
                self.current_batch.push(item);
            }
        }
    }
}
