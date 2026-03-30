//! Cycle detection and iterator utilities.

use std::iter::Iterator;

/// Takes items from an iterator until the predicate returns true.
pub fn take_until<I, P>(iter: I, predicate: P) -> Vec<I::Item>
where
    I: Iterator,
    P: Fn(&I::Item) -> bool + Clone,
{
    let mut result = Vec::new();
    for item in iter {
        if predicate(&item) {
            break;
        }
        result.push(item);
    }
    result
}
