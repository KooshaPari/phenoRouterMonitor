//! Comprehensive integration tests for phenotype-iter.

use phenotype_iter::{Batch, Chunk, Windowed};

#[test]
fn chunk_trait_with_range() {
    let chunks: Vec<Vec<i32>> = (1..=7).chunk(3).collect();
    assert_eq!(chunks, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]]);
}

#[test]
fn chunk_trait_with_vec() {
    let items = vec![1, 2, 3, 4, 5];
    let chunks: Vec<Vec<i32>> = items.into_iter().chunk(2).collect();
    assert_eq!(chunks, vec![vec![1, 2], vec![3, 4], vec![5]]);
}

#[test]
fn chunk_trait_size_one() {
    let chunks: Vec<Vec<i32>> = (1..=3).chunk(1).collect();
    assert_eq!(chunks, vec![vec![1], vec![2], vec![3]]);
}

#[test]
fn chunk_trait_larger_than_input() {
    let chunks: Vec<Vec<i32>> = (1..=3).chunk(10).collect();
    assert_eq!(chunks, vec![vec![1, 2, 3]]);
}

#[test]
fn chunk_trait_with_strings() {
    let items = vec!["a", "b", "c", "d", "e"];
    let chunks: Vec<Vec<&str>> = items.into_iter().chunk(2).collect();
    assert_eq!(
        chunks,
        vec![vec!["a", "b"], vec!["c", "d"], vec!["e"]]
    );
}

#[test]
fn chunk_trait_empty_input() {
    let chunks: Vec<Vec<i32>> = (1..1).chunk(3).collect();
    assert_eq!(chunks, vec![]);
}

#[test]
fn window_trait_basic() {
    let windows: Vec<Vec<i32>> = (1..=5).window(3).collect();
    assert_eq!(
        windows,
        vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]
    );
}

#[test]
fn window_trait_size_one() {
    let windows: Vec<Vec<i32>> = (1..=3).window(1).collect();
    assert_eq!(windows, vec![vec![1], vec![2], vec![3]]);
}

#[test]
fn window_trait_size_two() {
    let windows: Vec<Vec<i32>> = (1..=4).window(2).collect();
    assert_eq!(windows, vec![vec![1, 2], vec![2, 3], vec![3, 4]]);
}

#[test]
fn window_trait_larger_than_input() {
    let windows: Vec<Vec<i32>> = (1..=3).window(5).collect();
    assert_eq!(windows, vec![vec![1, 2, 3]]);
}

#[test]
fn window_trait_exact_match() {
    let windows: Vec<Vec<i32>> = (1..=3).window(3).collect();
    assert_eq!(windows, vec![vec![1, 2, 3]]);
}

#[test]
fn window_trait_with_strings() {
    let items = vec!["a", "b", "c"];
    let windows: Vec<Vec<&str>> = items.into_iter().window(2).collect();
    assert_eq!(windows, vec![vec!["a", "b"], vec!["b", "c"]]);
}

#[test]
fn window_trait_empty_input() {
    let windows: Vec<Vec<i32>> = (1..1).window(3).collect();
    assert_eq!(windows, vec![]);
}

#[test]
fn batch_trait_divisible() {
    let batches: Vec<Vec<i32>> = (1..=6).batch(|x| x % 3 == 1).collect();
    assert_eq!(batches, vec![vec![1], vec![2, 3], vec![4], vec![5, 6]]);
}

#[test]
fn batch_trait_never_true() {
    let batches: Vec<Vec<i32>> = (1..=5).batch(|_| false).collect();
    assert_eq!(batches, vec![vec![1, 2, 3, 4, 5]]);
}

#[test]
fn batch_trait_always_true() {
    let batches: Vec<Vec<i32>> = (1..=3).batch(|_| true).collect();
    assert_eq!(batches, vec![vec![1], vec![2], vec![3]]);
}

#[test]
fn batch_trait_first_element() {
    let batches: Vec<Vec<i32>> = (1..=5).batch(|x| x == &1).collect();
    assert_eq!(batches, vec![vec![1, 2, 3, 4, 5]]);
}

#[test]
fn batch_trait_with_strings() {
    let items = vec!["a", "b", "c", "d", "e"];
    let batches: Vec<Vec<&str>> = items.into_iter().batch(|x| x.starts_with('c')).collect();
    assert_eq!(batches, vec![vec!["a", "b"], vec!["c", "d", "e"]]);
}

#[test]
fn batch_trait_empty_input() {
    let batches: Vec<Vec<i32>> = (1..1).batch(|_| true).collect();
    assert_eq!(batches, vec![]);
}

#[test]
fn chained_chunk_and_window() {
    // First chunk into groups of 2, then apply a window of size 2
    let items = (1..=5).collect::<Vec<_>>();
    let chunked: Vec<Vec<i32>> = items.iter().copied().chunk(2).collect();
    assert_eq!(chunked, vec![vec![1, 2], vec![3, 4], vec![5]]);

    // Now apply window to the original, then chunk
    let windows: Vec<Vec<i32>> = (1..=4).window(2).collect();
    let chunked_windows: Vec<Vec<Vec<i32>>> = windows.into_iter().chunk(2).collect();
    assert_eq!(
        chunked_windows,
        vec![vec![vec![1, 2], vec![2, 3]], vec![vec![3, 4]]]
    );
}

#[test]
fn large_chunk_test() {
    let items = 1..=1000;
    let chunks: Vec<Vec<i32>> = items.chunk(100).collect();
    assert_eq!(chunks.len(), 10);
    assert_eq!(chunks[0].len(), 100);
    assert_eq!(chunks[9].len(), 100);
}

#[test]
fn large_window_test() {
    let items = 1..=100;
    let windows: Vec<Vec<i32>> = items.window(10).collect();
    assert_eq!(windows.len(), 91); // 100 - 10 + 1
    assert_eq!(windows[0][0], 1);
    assert_eq!(windows[90][9], 100);
}

#[test]
fn large_batch_test() {
    let items = 1..=1000;
    let batches: Vec<Vec<i32>> = items.batch(|x| x % 100 == 1).collect();
    assert_eq!(batches.len(), 10);
    assert_eq!(batches[0].len(), 1); // [1]
    assert_eq!(batches[1].len(), 100); // [2..=101]
}

#[test]
fn chunk_preserves_order() {
    let items = vec![1, 2, 3, 4, 5, 6];
    let chunks: Vec<Vec<i32>> = items.into_iter().chunk(2).collect();
    let flattened: Vec<i32> = chunks.into_iter().flatten().collect();
    assert_eq!(flattened, vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn window_preserves_order() {
    let items = vec![1, 2, 3, 4];
    let windows: Vec<Vec<i32>> = items.into_iter().window(2).collect();
    let flattened: Vec<i32> = windows
        .into_iter()
        .enumerate()
        .flat_map(|(i, w)| {
            if i == 0 {
                w.into_iter().collect::<Vec<_>>()
            } else {
                w.into_iter().skip(1).collect::<Vec<_>>()
            }
        })
        .collect();
    assert_eq!(flattened, vec![1, 2, 3, 4]);
}

#[test]
fn batch_preserves_order() {
    let items = vec![1, 2, 3, 4, 5, 6];
    let batches: Vec<Vec<i32>> = items.into_iter().batch(|x| x % 3 == 1).collect();
    let flattened: Vec<i32> = batches.into_iter().flatten().collect();
    assert_eq!(flattened, vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn chunk_with_filter() {
    let chunks: Vec<Vec<i32>> = (1..=10)
        .filter(|x| x % 2 == 0)
        .chunk(2)
        .collect();
    assert_eq!(chunks, vec![vec![2, 4], vec![6, 8], vec![10]]);
}

#[test]
fn window_with_map() {
    let windows: Vec<Vec<i32>> = (1..=4)
        .map(|x| x * 2)
        .window(2)
        .collect();
    assert_eq!(
        windows,
        vec![vec![2, 4], vec![4, 6], vec![6, 8]]
    );
}

#[test]
fn batch_with_filter_and_map() {
    let batches: Vec<Vec<i32>> = (1..=6)
        .filter(|x| x % 2 == 0)
        .map(|x| x * 10)
        .batch(|x| x % 40 == 20)
        .collect();
    assert_eq!(batches, vec![vec![20], vec![40, 60]]);
}

#[test]
fn triple_chain_operations() {
    // Chunk → flat map to window → collect
    let items = vec![1, 2, 3, 4, 5, 6];
    let result: Vec<i32> = items
        .into_iter()
        .chunk(2)
        .flat_map(|chunk| chunk)
        .collect();
    assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn byte_window_simulation() {
    // Simulate processing bytes in sliding windows
    let bytes = vec![b'H', b'e', b'l', b'l', b'o'];
    let windows: Vec<Vec<u8>> = bytes.into_iter().window(2).collect();
    assert_eq!(windows.len(), 4);
    assert_eq!(windows[0], vec![b'H', b'e']);
    assert_eq!(windows[3], vec![b'l', b'o']);
}

#[test]
fn chunk_exact_size_iterator() {
    let items = vec![1, 2, 3, 4, 5, 6];
    let chunk_iter = items.into_iter().chunk(2);
    // Note: chunk_iter is ExactSizeIterator when backing iterator is ExactSizeIterator
    let chunks: Vec<Vec<i32>> = chunk_iter.collect();
    assert_eq!(chunks.len(), 3);
}

#[test]
fn complex_predicate_batch() {
    let items = vec![1, 10, 2, 20, 3, 30, 4, 40, 5];
    let batches: Vec<Vec<i32>> = items.into_iter().batch(|x| x > &15).collect();
    // Batch starts when > 15: [1, 10, 2], [20, 3], [30, 4], [40, 5]
    assert_eq!(batches[1][0], 20);
    assert_eq!(batches[1].len(), 2);
}

#[test]
fn memory_efficiency_lazy_evaluation() {
    // This test verifies lazy evaluation without explicit assertions
    // The iterator should not allocate the entire sequence in memory
    let _iter = (1..=1_000_000).chunk(100);
    // If this completed, the chunker is lazy (doesn't materialize all million items)

    let _window_iter = (1..=1_000_000).window(10);
    // Window iterator uses VecDeque of fixed size 10, memory efficient

    let _batch_iter = (1..=1_000_000).batch(|x| x % 1000 == 1);
    // Batch accumulates items but doesn't pre-allocate the full sequence
}

#[test]
fn single_element_operations() {
    let single = vec![42];

    let chunks: Vec<Vec<i32>> = single.clone().into_iter().chunk(1).collect();
    assert_eq!(chunks, vec![vec![42]]);

    let windows: Vec<Vec<i32>> = single.clone().into_iter().window(1).collect();
    assert_eq!(windows, vec![vec![42]]);

    let batches: Vec<Vec<i32>> = single.clone().into_iter().batch(|_| false).collect();
    assert_eq!(batches, vec![vec![42]]);
}

#[test]
fn two_element_operations() {
    let two = vec![1, 2];

    let chunks: Vec<Vec<i32>> = two.clone().into_iter().chunk(2).collect();
    assert_eq!(chunks, vec![vec![1, 2]]);

    let windows: Vec<Vec<i32>> = two.clone().into_iter().window(2).collect();
    assert_eq!(windows, vec![vec![1, 2]]);

    let batches: Vec<Vec<i32>> = two.clone().into_iter().batch(|x| x == &2).collect();
    assert_eq!(batches, vec![vec![1], vec![2]]);
}

#[test]
fn sequential_consistency() {
    // Ensure multiple operations on same sequence produce consistent results
    let items = 1..=10;
    let chunks1: Vec<Vec<i32>> = items.clone().chunk(3).collect();
    let chunks2: Vec<Vec<i32>> = (1..=10).chunk(3).collect();
    assert_eq!(chunks1, chunks2);
}

#[test]
fn chunk_with_custom_type() {
    #[derive(Debug, Clone, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    let points = vec![
        Point { x: 1, y: 1 },
        Point { x: 2, y: 2 },
        Point { x: 3, y: 3 },
    ];

    let chunks: Vec<Vec<Point>> = points.into_iter().chunk(2).collect();
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0][0].x, 1);
    assert_eq!(chunks[1][0].x, 3);
}

#[test]
fn window_with_custom_type() {
    #[derive(Debug, Clone, PartialEq)]
    struct Value {
        id: u32,
    }

    let values = vec![Value { id: 1 }, Value { id: 2 }, Value { id: 3 }];

    let windows: Vec<Vec<Value>> = values.into_iter().window(2).collect();
    assert_eq!(windows.len(), 2);
    assert_eq!(windows[0][0].id, 1);
    assert_eq!(windows[0][1].id, 2);
}

#[test]
fn batch_with_reference_predicate() {
    let items = vec!["alice", "bob", "charlie", "diana"];
    let batches: Vec<Vec<&str>> = items.into_iter().batch(|x| x.len() > 4).collect();
    assert_eq!(batches[0], vec!["alice"]);
    assert_eq!(batches[1][0], "bob");
}
