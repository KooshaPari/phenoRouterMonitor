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
    assert_eq!(chunks, vec![] as Vec<Vec<i32>>);
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
    assert_eq!(windows, vec![] as Vec<Vec<i32>>);
}

#[test]
fn batch_trait_divisible() {
    // When predicate is true (x % 3 == 1), a new batch starts
    // Predicate true for 1 and 4
    let batches: Vec<Vec<i32>> = (1..=6).batch(|x| x % 3 == 1).collect();
    assert_eq!(batches, vec![vec![1, 2, 3], vec![4, 5, 6]]);
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
    assert_eq!(batches, vec![] as Vec<Vec<i32>>);
}

#[test]
fn chained_operations() {
    let items = (1..=5).collect::<Vec<_>>();
    let chunked: Vec<Vec<i32>> = items.iter().copied().chunk(2).collect();
    assert_eq!(chunked, vec![vec![1, 2], vec![3, 4], vec![5]]);
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
    assert_eq!(windows.len(), 91);
    assert_eq!(windows[0][0], 1);
    assert_eq!(windows[90][9], 100);
}

#[test]
fn large_batch_test() {
    // Predicate x % 100 == 1 is true for 1, 101, 201, ..., 901
    let items = 1..=1000;
    let batches: Vec<Vec<i32>> = items.batch(|x| x % 100 == 1).collect();
    // Should have 10 batches: [1..100], [101..200], [201..300], ..., [901..1000]
    assert_eq!(batches.len(), 10);
    assert_eq!(batches[0].len(), 100);
    assert_eq!(batches[1].len(), 100);
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
    // After filter: [2, 4, 6], after map: [20, 40, 60]
    // Predicate x % 40 == 20 is true for 20 and 60
    let batches: Vec<Vec<i32>> = (1..=6)
        .filter(|x| x % 2 == 0)
        .map(|x| x * 10)
        .batch(|x| x % 40 == 20)
        .collect();
    assert_eq!(batches, vec![vec![20, 40], vec![60]]);
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
    let items = 1..=10;
    let chunks1: Vec<Vec<i32>> = items.clone().chunk(3).collect();
    let chunks2: Vec<Vec<i32>> = (1..=10).chunk(3).collect();
    assert_eq!(chunks1, chunks2);
}

#[test]
fn byte_window_simulation() {
    let bytes = vec![b'H', b'e', b'l', b'l', b'o'];
    let windows: Vec<Vec<u8>> = bytes.into_iter().window(2).collect();
    assert_eq!(windows.len(), 4);
    assert_eq!(windows[0], vec![b'H', b'e']);
    assert_eq!(windows[3], vec![b'l', b'o']);
}

#[test]
fn memory_efficiency_lazy_evaluation() {
    // These should not cause panic or excessive memory usage
    let _iter = (1..=1_000_000).chunk(100);
    let _window_iter = (1..=1_000_000).window(10);
    let _batch_iter = (1..=1_000_000).batch(|x| x % 1000 == 1);
}

#[test]
fn batch_with_reference_predicate() {
    // alice (5 chars > 4), bob (3 chars), charlie (7 chars > 4), diana (5 chars > 4)
    // Predicate x.len() > 4 is true for alice, charlie, diana
    let items = vec!["alice", "bob", "charlie", "diana"];
    let batches: Vec<Vec<&str>> = items.into_iter().batch(|x| x.len() > 4).collect();
    // Batches: [alice, bob], [charlie], [diana]
    assert_eq!(batches[0], vec!["alice", "bob"]);
    assert_eq!(batches[1], vec!["charlie"]);
    assert_eq!(batches[2], vec!["diana"]);
}
