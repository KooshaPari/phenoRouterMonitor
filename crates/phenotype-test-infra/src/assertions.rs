//! Custom assertion macros.

#[macro_export]
macro_rules! assert_contains {
    ($haystack:expr, $needle:expr) => {
        let h = &($haystack);
        let n = $needle;
        if !h.contains(n) {
            panic!("assert_contains failed: {:?} does not contain {:?}", h, n);
        }
    };
}

#[macro_export]
macro_rules! assert_is_empty {
    ($collection:expr) => {
        if !$collection.is_empty() {
            panic!("assert_is_empty failed: collection not empty");
        }
    };
}

#[macro_export]
macro_rules! assert_not_empty {
    ($collection:expr) => {
        if $collection.is_empty() {
            panic!("assert_not_empty failed: collection is empty");
        }
    };
}

#[macro_export]
macro_rules! assert_len {
    ($collection:expr, $expected:expr) => {
        if $collection.len() != $expected {
            panic!("assert_len failed: expected {}, got {}", $expected, $collection.len());
        }
    };
}

#[macro_export]
macro_rules! assert_all {
    ($collection:expr, $predicate:expr) => {
        for item in &$collection {
            if !$predicate(item) {
                panic!("assert_all failed: predicate returned false");
            }
        }
    };
}

#[macro_export]
macro_rules! assert_any {
    ($collection:expr, $predicate:expr) => {
        let found = $collection.iter().any($predicate);
        if !found {
            panic!("assert_any failed: no item satisfied predicate");
        }
    };
}
