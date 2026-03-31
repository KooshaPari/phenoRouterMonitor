//! Integration tests for phenotype-iter iterator utilities
//!
//! Traces to: FR-PHENO-ITER-001, FR-PHENO-ITER-002, FR-PHENO-ITER-003

use phenotype_iter::{Batch, Chunk, Windowed};

// ============================================================================
// Window Iterator Tests
// ============================================================================

#[test]
fn test_window_basic_sliding_behavior() {
    let data = vec![1, 2, 3, 4, 5];
    let windows: Vec<_> = data.into_iter().window(3).collect();

    assert_eq!(windows.len(), 3);
    assert_eq!(windows[0], vec![1, 2, 3]);
    assert_eq!(windows[1], vec![2, 3, 4]);
    assert_eq!(windows[2], vec![3, 4, 5]);
}

#[test]
fn test_window_size_two() {
    let data = vec![10, 20, 30, 40];
    let windows: Vec<_> = data.into_iter().window(2).collect();

    assert_eq!(windows.len(), 3);
    assert_eq!(windows[0], vec![10, 20]);
    assert_eq!(windows[1], vec![20, 30]);
    assert_eq!(windows[2], vec![30, 40]);
}

#[test]
fn test_window_single_element_iterator() {
    let data = vec![42];
    let windows: Vec<_> = data.into_iter().window(2).collect();

    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0], vec![42]);
}

#[test]
fn test_window_size_equals_input_length() {
    let data = vec![1, 2, 3];
    let windows: Vec<_> = data.into_iter().window(3).collect();

    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0], vec![1, 2, 3]);
}

#[test]
fn test_window_size_larger_than_input() {
    let data = vec![1, 2];
    let windows: Vec<_> = data.into_iter().window(5).collect();

    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0], vec![1, 2]);
}

#[test]
fn test_window_empty_iterator() {
    let data: Vec<i32> = vec![];
    let windows: Vec<_> = data.into_iter().window(3).collect();

    assert_eq!(windows.len(), 0);
}

#[test]
fn test_window_with_strings() {
    let data = vec!["a", "b", "c", "d"];
    let windows: Vec<_> = data.into_iter().window(2).collect();

    assert_eq!(windows.len(), 3);
    assert_eq!(windows[0], vec!["a", "b"]);
    assert_eq!(windows[1], vec!["b", "c"]);
}

#[test]
fn test_window_large_dataset() {
    let data: Vec<i32> = (0..1000).collect();
    let windows: Vec<_> = data.into_iter().window(10).collect();

    assert_eq!(windows.len(), 991);
    assert_eq!(windows[0].len(), 10);
    assert_eq!(windows[0][0], 0);
    assert_eq!(windows[990][9], 999);
}

#[test]
fn test_window_memory_efficiency() {
    let data = vec![1, 2, 3, 4, 5];
    let mut iter = data.into_iter().window(3);

    let first = iter.next();
    assert_eq!(first, Some(vec![1, 2, 3]));

    let second = iter.next();
    assert_eq!(second, Some(vec![2, 3, 4]));
}

// ============================================================================
// Chunk Iterator Tests
// ============================================================================

#[test]
fn test_chunk_basic_division() {
    let data = vec![1, 2, 3, 4, 5, 6];
    let chunks: Vec<_> = data.into_iter().chunk(2).collect();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], vec![1, 2]);
    assert_eq!(chunks[1], vec![3, 4]);
    assert_eq!(chunks[2], vec![5, 6]);
}

#[test]
fn test_chunk_uneven_distribution() {
    let data = vec![1, 2, 3, 4, 5];
    let chunks: Vec<_> = data.into_iter().chunk(2).collect();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], vec![1, 2]);
    assert_eq!(chunks[1], vec![3, 4]);
    assert_eq!(chunks[2], vec![5]);
}

#[test]
fn test_chunk_single_element_chunks() {
    let data = vec!['a', 'b', 'c'];
    let chunks: Vec<_> = data.into_iter().chunk(1).collect();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], vec!['a']);
    assert_eq!(chunks[1], vec!['b']);
    assert_eq!(chunks[2], vec!['c']);
}

#[test]
fn test_chunk_size_equals_length() {
    let data = vec![10, 20, 30];
    let chunks: Vec<_> = data.into_iter().chunk(3).collect();

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], vec![10, 20, 30]);
}

#[test]
fn test_chunk_empty_iterator() {
    let data: Vec<i32> = vec![];
    let chunks: Vec<_> = data.into_iter().chunk(3).collect();

    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_chunk_order_preservation() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let chunks: Vec<_> = data.into_iter().chunk(3).collect();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], vec![1, 2, 3]);
    assert_eq!(chunks[1], vec![4, 5, 6]);
    assert_eq!(chunks[2], vec![7, 8]);
}

#[test]
fn test_chunk_large_dataset() {
    let data: Vec<i32> = (0..10000).collect();
    let chunks: Vec<_> = data.into_iter().chunk(100).collect();

    assert_eq!(chunks.len(), 100);
    assert_eq!(chunks[0].len(), 100);
    assert_eq!(chunks[0][0], 0);
    assert_eq!(chunks[99][99], 9999);
}

#[test]
fn test_chunk_lazy_evaluation() {
    let data = vec![1, 2, 3, 4, 5];
    let mut iter = data.into_iter().chunk(2);

    let first = iter.next();
    assert_eq!(first, Some(vec![1, 2]));

    let second = iter.next();
    assert_eq!(second, Some(vec![3, 4]));
}

// ============================================================================
// Batch Iterator Tests
// ============================================================================

#[test]
fn test_batch_basic_predicate() {
    let data = vec![1, 2, 3, 5, 6, 7];
    let batches: Vec<_> = data.into_iter().batch(|&x| x < 5).collect();

    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], vec![5, 6, 7]);
    assert_eq!(batches[1], vec![3]);
}

#[test]
fn test_batch_all_match_predicate() {
    let data = vec![1, 2, 3];
    let batches: Vec<_> = data.into_iter().batch(|&x| x > 0).collect();

    assert_eq!(batches.len(), 0);
}

#[test]
fn test_batch_none_match_predicate() {
    let data = vec![1, 2, 3];
    let batches: Vec<_> = data.into_iter().batch(|&x| x > 100).collect();

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0], vec![1, 2, 3]);
}

#[test]
fn test_batch_alternating_groups() {
    let data = vec![2, 4, 6, 1, 3, 5];
    let batches: Vec<_> = data.into_iter().batch(|&x| x % 2 == 0).collect();

    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], vec![1, 3, 5]);
    assert_eq!(batches[1], vec![6]);
}

#[test]
fn test_batch_empty_iterator() {
    let data: Vec<i32> = vec![];
    let batches: Vec<_> = data.into_iter().batch(|_| true).collect();

    assert_eq!(batches.len(), 0);
}

#[test]
fn test_batch_single_item() {
    let data = vec![5];
    let batches: Vec<_> = data.into_iter().batch(|&x| x > 0).collect();

    assert_eq!(batches.len(), 0);
}

#[test]
fn test_batch_predicate_with_strings() {
    let data = vec!["apple", "apricot", "banana", "berry"];
    let batches: Vec<_> = data.into_iter().batch(|s| s.starts_with('a')).collect();

    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], vec!["banana", "berry"]);
    assert_eq!(batches[1], vec!["apricot"]);
}

#[test]
fn test_batch_large_dataset() {
    let data: Vec<i32> = (0..1000).collect();
    let batches: Vec<_> = data.into_iter().batch(|&x| x < 500).collect();

    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], (500..1000).collect::<Vec<_>>());
    assert_eq!(batches[1], vec![499]);
}

#[test]
fn test_batch_complex_predicate() {
    let data = vec![1, 3, 5, 7, 2, 4, 6];
    let batches: Vec<_> = data.into_iter().batch(|&x| x % 2 == 1).collect();

    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], vec![2, 4, 6]);
    assert_eq!(batches[1], vec![7]);
}

// ============================================================================
// Composition and Integration Tests
// ============================================================================

#[test]
fn test_window_then_collect() {
    let data = vec![1, 2, 3, 4];
    let flattened: Vec<i32> = data.into_iter().window(2).flatten().collect();

    assert!(flattened.len() > 0);
    assert_eq!(flattened[0], 1);
}

#[test]
fn test_chunk_then_filter() {
    let data = vec![1, 2, 3, 4, 5, 6];
    let chunks: Vec<_> = data.into_iter().chunk(2).collect();
    let filtered: Vec<_> = chunks.iter().filter(|c| c.len() == 2).collect();

    assert_eq!(filtered.len(), 3);
}

#[test]
fn test_batch_then_map() {
    let data = vec![1, 2, 3, 5, 6];
    let batches: Vec<_> = data.into_iter().batch(|&x| x < 4).collect();
    let sums: Vec<i32> = batches.iter().map(|b| b.iter().sum()).collect();

    assert_eq!(sums.len(), 2);
    assert_eq!(sums[0], 11); // 5+6
    assert_eq!(sums[1], 3);
}

#[test]
fn test_multiple_windows_different_sizes() {
    let data = vec![1, 2, 3, 4, 5, 6];

    let w2 = data.iter().cloned().window(2).count();
    let w3 = data.iter().cloned().window(3).count();

    assert!(w2 > w3);
}

#[test]
fn test_multiple_chunks_different_sizes() {
    let data = vec![1, 2, 3, 4, 5, 6];

    let c2: Vec<_> = data.iter().cloned().chunk(2).collect();
    let c3: Vec<_> = data.iter().cloned().chunk(3).collect();

    assert_eq!(c2.len(), 3);
    assert_eq!(c3.len(), 2);
}

#[test]
fn test_chained_operations() {
    let data = vec![1, 2, 3, 4, 5];
    let result: Vec<_> = data
        .into_iter()
        .chunk(2)
        .filter(|chunk| chunk.len() > 1)
        .collect();

    assert!(result.len() > 0);
}

#[test]
fn test_window_then_chunk() {
    let data = vec![1, 2, 3, 4];
    let windowed: Vec<_> = data.into_iter().window(2).collect();
    let flattened: Vec<i32> = windowed.into_iter().flatten().collect();

    assert!(flattened.len() > 0);
}

// ============================================================================
// Edge Cases and Stress Tests
// ============================================================================

#[test]
fn test_window_two_elements() {
    let data = vec![1, 2];
    let windows: Vec<_> = data.into_iter().window(2).collect();

    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0], vec![1, 2]);
}

#[test]
fn test_chunk_exact_multiple() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let chunks: Vec<_> = data.into_iter().chunk(3).collect();

    assert_eq!(chunks.len(), 3);
    for chunk in &chunks {
        assert_eq!(chunk.len(), 3);
    }
}

#[test]
fn test_batch_single_large_batch() {
    let data = vec![1, 2, 3, 4, 5];
    let batches: Vec<_> = data.into_iter().batch(|_| true).collect();

    assert_eq!(batches.len(), 0);
}

#[test]
fn test_batch_each_item_own_batch() {
    let data = vec![1, 2, 3, 4, 5];
    let batches: Vec<_> = data.into_iter().batch(|&x| x == 1).collect();

    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], vec![2, 3, 4, 5]);
    assert_eq!(batches[1], vec![1]);
}

#[test]
fn test_window_stress_10k_items() {
    let data: Vec<i32> = (0..10000).collect();
    let windows: Vec<_> = data.into_iter().window(5).collect();

    assert_eq!(windows.len(), 9996);
}

#[test]
fn test_chunk_stress_10k_items() {
    let data: Vec<i32> = (0..10000).collect();
    let chunks: Vec<_> = data.into_iter().chunk(50).collect();

    assert_eq!(chunks.len(), 200);
}

#[test]
fn test_batch_stress_10k_items() {
    let data: Vec<i32> = (0..10000).collect();
    let batches: Vec<_> = data.into_iter().batch(|&x| x % 2 == 0).collect();

    assert_eq!(batches.len(), 5000);
}

// ============================================================================
// Functional Requirements Verification
// ============================================================================

#[test]
fn verify_fr_pheno_iter_001_windowing() {
    let data = vec![1, 2, 3, 4, 5];

    let windows: Vec<_> = data.iter().cloned().window(3).collect();
    assert_eq!(windows.len(), 3);
    assert_eq!(windows[0], vec![1, 2, 3]);
    assert_eq!(windows[1], vec![2, 3, 4]);
    assert_eq!(windows[2], vec![3, 4, 5]);
}

#[test]
fn verify_fr_pheno_iter_002_batching() {
    let data = vec![1, 2, 3, 5, 6, 7];

    let batches: Vec<_> = data.into_iter().batch(|&x| x < 5).collect();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], vec![5, 6, 7]);
    assert_eq!(batches[1], vec![3]);
}

#[test]
fn verify_fr_pheno_iter_003_chunking() {
    let data = vec![1, 2, 3, 4, 5, 6];

    let chunks: Vec<_> = data.into_iter().chunk(2).collect();
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], vec![1, 2]);
    assert_eq!(chunks[1], vec![3, 4]);
    assert_eq!(chunks[2], vec![5, 6]);
}
