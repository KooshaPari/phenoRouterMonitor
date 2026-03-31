
// ============================================================================
// Batch Iterator Tests  
// ============================================================================

#[test]
fn test_batch_predicate_true_yields_batch() {
    let data = vec![1, 2, 3, 4, 5, 6];
    let batches: Vec<_> = data.into_iter().batch_by(|x| *x >= 4).collect();
    assert_eq!(batches, vec![vec![1, 2, 3], vec![4, 5, 6]]);
}

#[test]
fn test_batch_all_false() {
    let data = vec![1, 2, 3];
    let batches: Vec<_> = data.into_iter().batch_by(|x| *x > 10).collect();
    assert_eq!(batches, vec![vec![1, 2, 3]]);
}

#[test]
fn test_batch_empty() {
    let data: Vec<i32> = vec![];
    let batches: Vec<_> = data.into_iter().batch_by(|x| *x > 0).collect();
    assert!(batches.is_empty());
}
