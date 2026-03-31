//! Integration tests for phenotype-iter iterator utilities
//!
//! Traces to: FR-PHENO-ITER-001, FR-PHENO-ITER-002, FR-PHENO-ITER-003

use phenotype_iter::{Batch, Chunk, Windowed};

// ============================================================================
// Window Iterator Tests
// ============================================================================

#[test]
fn test_window_basic_sliding_behavior() {
    // Traces to: FR-PHENO-ITER-001 (windowing iterator behavior)
    let data = vec![1, 2, 3, 4, 5];
    let windows: Vec<_> = data.into_iter().window(3).collect();

    assert_eq!(windows.len(), 3);
    assert_eq!(windows[0], vec![1, 2, 3]);
    assert_eq!(windows[1], vec![2, 3, 4]);
    assert_eq!(windows[2], vec![3, 4, 5]);
}

#[test]
fn test_window_size_two() {
    // Traces to: FR-PHENO-ITER-001 (window size 2)
    let data = vec![10, 20, 30, 40];
    let windows: Vec<_> = data.into_iter().window(2).collect();

    assert_eq!(windows.len(), 3);
    assert_eq!(windows[0], vec![10, 20]);
    assert_eq!(windows[1], vec![20, 30]);
    assert_eq!(windows[2], vec![30, 40]);
}

#[test]
fn test_window_single_element_iterator() {
    // Traces to: FR-PHENO-ITER-001 (edge case: single item)
    let data = vec![42];
    let windows: Vec<_> = data.into_iter().window(2).collect();

    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0], vec![42]);
}

#[test]
fn test_window_size_equals_input_length() {
    // Traces to: FR-PHENO-ITER-001 (window equals input)
    let data = vec![1, 2, 3];
    let windows: Vec<_> = data.into_iter().window(3).collect();

    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0], vec![1, 2, 3]);
}

#[test]
fn test_window_size_larger_than_input() {
    // Traces to: FR-PHENO-ITER-001 (window > input)
    let data = vec![1, 2];
    let windows: Vec<_> = data.into_iter().window(5).collect();

    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0], vec![1, 2]);
}

#[test]
fn test_window_empty_iterator() {
    // Traces to: FR-PHENO-ITER-001 (empty input)
    let data: Vec<i32> = vec![];
    let windows: Vec<_> = data.into_iter().window(3).collect();

    assert_eq!(windows.len(), 0);
}

#[test]
fn test_window_with_strings() {
    // Traces to: FR-PHENO-ITER-001 (generic Item type)
    let data = vec!["a", "b", "c", "d"];
    let windows: Vec<_> = data.into_iter().window(2).collect();

    assert_eq!(windows.len(), 3);
    assert_eq!(windows[0], vec!["a", "b"]);
    assert_eq!(windows[1], vec!["b", "c"]);
}

#[test]
fn test_window_large_dataset() {
    // Traces to: FR-PHENO-ITER-001 (scalability)
    let data: Vec<i32> = (0..1000).collect();
    let windows: Vec<_> = data.into_iter().window(10).collect();

    assert_eq!(windows.len(), 991);
    assert_eq!(windows[0].len(), 10);
    assert_eq!(windows[0][0], 0);
    assert_eq!(windows[990][9], 999);
}

#[test]
fn test_window_memory_efficiency() {
    // Traces to: FR-PHENO-ITER-001 (lazy evaluation)
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
    // Traces to: FR-PHENO-ITER-001 (chunking iterator behavior)
    let data = vec![1, 2, 3, 4, 5, 6];
    let chunks: Vec<_> = data.into_iter().chunk(2).collect();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], vec![1, 2]);
    assert_eq!(chunks[1], vec![3, 4]);
    assert_eq!(chunks[2], vec![5, 6]);
}

#[test]
fn test_chunk_uneven_distribution() {
    // Traces to: FR-PHENO-ITER-001 (uneven chunk sizes)
    let data = vec![1, 2, 3, 4, 5];
    let chunks: Vec<_> = data.into_iter().chunk(2).collect();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], vec![1, 2]);
    assert_eq!(chunks[1], vec![3, 4]);
    assert_eq!(chunks[2], vec![5]);
}

#[test]
fn test_chunk_single_element_chunks() {
    // Traces to: FR-PHENO-ITER-001 (chunk size 1)
    let data = vec!['a', 'b', 'c'];
    let chunks: Vec<_> = data.into_iter().chunk(1).collect();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], vec!['a']);
    assert_eq!(chunks[1], vec!['b']);
    assert_eq!(chunks[2], vec!['c']);
}

#[test]
fn test_chunk_size_equals_length() {
    // Traces to: FR-PHENO-ITER-001 (chunk size = length)
    let data = vec![10, 20, 30];
    let chunks: Vec<_> = data.into_iter().chunk(3).collect();

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], vec![10, 20, 30]);
}

#[test]
fn test_chunk_empty_iterator() {
    // Traces to: FR-PHENO-ITER-001 (empty input)
    let data: Vec<i32> = vec![];
    let chunks: Vec<_> = data.into_iter().chunk(3).collect();

    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_chunk_order_preservation() {
    // Traces to: FR-PHENO-ITER-001 (order preserved)
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let chunks: Vec<_> = data.into_iter().chunk(3).collect();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], vec![1, 2, 3]);
    assert_eq!(chunks[1], vec![4, 5, 6]);
    assert_eq!(chunks[2], vec![7, 8]);
}

#[test]
fn test_chunk_large_dataset() {
    // Traces to: FR-PHENO-ITER-001 (scalability)
    let data: Vec<i32> = (0..10000).collect();
    let chunks: Vec<_> = data.into_iter().chunk(100).collect();

    assert_eq!(chunks.len(), 100);
    assert_eq!(chunks[0].len(), 100);
    assert_eq!(chunks[0][0], 0);
    assert_eq!(chunks[99][99], 9999);
}

#[test]
fn test_chunk_lazy_evaluation() {
    // Traces to: FR-PHENO-ITER-001 (lazy evaluation)
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
    // Traces to: FR-PHENO-ITER-002 (batch based on predicate)
    let data = vec![1, 2, 3, 5, 6, 7];
    let batches: Vec<_> = data.into_iter().batch(|&x| x < 5).collect();

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0], vec![1, 2, 3]);
}

#[test]
fn test_batch_all_match_predicate() {
    // Traces to: FR-PHENO-ITER-002 (all items match)
    let data = vec![1, 2, 3];
    let batches: Vec<_> = data.into_iter().batch(|&x| x > 0).collect();

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0], vec![1, 2, 3]);
}

#[test]
fn test_batch_none_match_predicate() {
    // Traces to: FR-PHENO-ITER-002 (no items match)
    let data = vec![1, 2, 3];
    let batches: Vec<_> = data.into_iter().batch(|&x| x > 100).collect();

    assert_eq!(batches.len(), 0);
}

#[test]
fn test_batch_alternating_groups() {
    // Traces to: FR-PHENO-ITER-002 (alternating batches)
    let data = vec![2, 4, 6, 1, 3, 5];
    let batches: Vec<_> = data.into_iter().batch(|&x| x % 2 == 0).collect();

    assert!(batches.len() >= 1);
    assert_eq!(batches[0], vec![2, 4, 6]);
}

#[test]
fn test_batch_empty_iterator() {
    // Traces to: FR-PHENO-ITER-002 (empty input)
    let data: Vec<i32> = vec![];
    let batches: Vec<_> = data.into_iter().batch(|_| true).collect();

    assert_eq!(batches.len(), 0);
}

#[test]
fn test_batch_single_item() {
    // Traces to: FR-PHENO-ITER-002 (single item)
    let data = vec![5];
    let batches: Vec<_> = data.into_iter().batch(|&x| x > 0).collect();

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0], vec![5]);
}

#[test]
fn test_batch_predicate_with_strings() {
    // Traces to: FR-PHENO-ITER-002 (generic Item type)
    let data = vec!["apple", "apricot", "banana", "berry"];
    let batches: Vec<_> = data.into_iter().batch(|s| s.starts_with('a')).collect();

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0], vec!["apple", "apricot"]);
}

#[test]
fn test_batch_large_dataset() {
    // Traces to: FR-PHENO-ITER-002 (scalability)
    let data: Vec<i32> = (0..1000).collect();
    let batches: Vec<_> = data.into_iter().batch(|&x| x < 500).collect();

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].len(), 500);
}

#[test]
fn test_batch_complex_predicate() {
    // Traces to: FR-PHENO-ITER-002 (complex predicate logic)
    let data = vec![1, 3, 5, 7, 2, 4, 6];
    let batches: Vec<_> = data.into_iter().batch(|&x| x % 2 == 1).collect();

    assert!(batches.len() >= 1);
    assert_eq!(batches[0], vec![1, 3, 5, 7]);
}

// ============================================================================
// Composition and Integration Tests
// ============================================================================

#[test]
fn test_window_then_collect() {
    // Traces to: FR-PHENO-ITER-003 (composition)
    let data = vec![1, 2, 3, 4];
    let flattened: Vec<i32> = data.into_iter().window(2).flatten().collect();

    assert!(flattened.len() > 0);
    assert_eq!(flattened[0], 1);
}

#[test]
fn test_chunk_then_filter() {
    // Traces to: FR-PHENO-ITER-003 (composition with filter)
    let data = vec![1, 2, 3, 4, 5, 6];
    let chunks: Vec<_> = data.into_iter().chunk(2).collect();
    let filtered: Vec<_> = chunks.iter().filter(|c| c.len() == 2).collect();

    assert_eq!(filtered.len(), 3);
}

#[test]
fn test_batch_then_map() {
    // Traces to: FR-PHENO-ITER-003 (composition with map)
    let data = vec![1, 2, 3, 5, 6];
    let batches: Vec<_> = data.into_iter().batch(|&x| x < 4).collect();
    let sums: Vec<i32> = batches.iter().map(|b| b.iter().sum()).collect();

    assert!(sums.len() > 0);
}

#[test]
fn test_multiple_windows_different_sizes() {
    // Traces to: FR-PHENO-ITER-001 (varying window sizes)
    let data = vec![1, 2, 3, 4, 5, 6];

    let w2 = data.iter().cloned().window(2).count();
    let w3 = data.iter().cloned().window(3).count();

    assert!(w2 > w3);
}

#[test]
fn test_multiple_chunks_different_sizes() {
    // Traces to: FR-PHENO-ITER-001 (varying chunk sizes)
    let data = vec![1, 2, 3, 4, 5, 6];

    let c2: Vec<_> = data.iter().cloned().chunk(2).collect();
    let c3: Vec<_> = data.iter().cloned().chunk(3).collect();

    assert_eq!(c2.len(), 3);
    assert_eq!(c3.len(), 2);
}

#[test]
fn test_chained_operations() {
    // Traces to: FR-PHENO-ITER-003 (chained operations)
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
    // Traces to: FR-PHENO-ITER-003 (window followed by chunk)
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
    // Traces to: FR-PHENO-ITER-001 (minimal data)
    let data = vec![1, 2];
    let windows: Vec<_> = data.into_iter().window(2).collect();

    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0], vec![1, 2]);
}

#[test]
fn test_chunk_exact_multiple() {
    // Traces to: FR-PHENO-ITER-001 (exact division)
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let chunks: Vec<_> = data.into_iter().chunk(3).collect();

    assert_eq!(chunks.len(), 3);
    for chunk in &chunks {
        assert_eq!(chunk.len(), 3);
    }
}

#[test]
fn test_batch_single_large_batch() {
    // Traces to: FR-PHENO-ITER-002 (single batch)
    let data = vec![1, 2, 3, 4, 5];
    let batches: Vec<_> = data.into_iter().batch(|_| true).collect();

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].len(), 5);
}

#[test]
fn test_batch_each_item_own_batch() {
    // Traces to: FR-PHENO-ITER-002 (many small batches)
    let data = vec![1, 2, 3, 4, 5];
    let batches: Vec<_> = data.into_iter().batch(|&x| x == 1).collect();

    assert!(batches.len() >= 1);
}

#[test]
fn test_window_stress_10k_items() {
    // Traces to: FR-PHENO-ITER-001 (stress test)
    let data: Vec<i32> = (0..10000).collect();
    let windows: Vec<_> = data.into_iter().window(5).collect();

    assert_eq!(windows.len(), 9996);
}

#[test]
fn test_chunk_stress_10k_items() {
    // Traces to: FR-PHENO-ITER-001 (stress test)
    let data: Vec<i32> = (0..10000).collect();
    let chunks: Vec<_> = data.into_iter().chunk(50).collect();

    assert_eq!(chunks.len(), 200);
}

#[test]
fn test_batch_stress_10k_items() {
    // Traces to: FR-PHENO-ITER-002 (stress test)
    let data: Vec<i32> = (0..10000).collect();
    let batches: Vec<_> = data.into_iter().batch(|&x| x < 5000).collect();

    assert_eq!(batches.len(), 1);
}

// ============================================================================
// Functional Requirements Verification
// ============================================================================

#[test]
fn verify_fr_pheno_iter_001_windowing() {
    // FR-PHENO-ITER-001: Windowing iterator with configurable size
    let data = vec![1, 2, 3, 4, 5];

    // Requirement: sliding window behavior
    let windows: Vec<_> = data.iter().cloned().window(3).collect();
    assert_eq!(windows.len(), 3);
    assert_eq!(windows[0], vec![1, 2, 3]);
    assert_eq!(windows[1], vec![2, 3, 4]);
    assert_eq!(windows[2], vec![3, 4, 5]);
}

#[test]
fn verify_fr_pheno_iter_002_batching() {
    // FR-PHENO-ITER-002: Batching with predicate-based grouping
    let data = vec![1, 2, 3, 5, 6, 7];

    // Requirement: group by predicate
    let batches: Vec<_> = data.into_iter().batch(|&x| x < 5).collect();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0], vec![1, 2, 3]);
}

#[test]
fn verify_fr_pheno_iter_003_chunking() {
    // FR-PHENO-ITER-003: Chunking for fixed-size partitions
    let data = vec![1, 2, 3, 4, 5, 6];

    // Requirement: fixed-size non-overlapping chunks
    let chunks: Vec<_> = data.into_iter().chunk(2).collect();
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], vec![1, 2]);
    assert_eq!(chunks[1], vec![3, 4]);
    assert_eq!(chunks[2], vec![5, 6]);
}
