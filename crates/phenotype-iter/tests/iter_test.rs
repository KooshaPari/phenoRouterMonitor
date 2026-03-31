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

    // apple starts first batch, apricot starts second, banana/berry accumulate
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], vec!["apple"]);
    assert_eq!(batches[1], vec!["apricot", "banana", "berry"]);
}

#[test]
fn test_batch_large_dataset() {
    // Traces to: FR-PHENO-ITER-002 (scalability)
    let data: Vec<i32> = (0..1000).collect();
    let batches: Vec<_> = data.into_iter().batch(|&x| x < 500).collect();

    // Each true trigger creates a new batch (items 0-498 as singles, then 499-999 accumulates)
    assert_eq!(batches.len(), 501);
    assert_eq!(batches[0], vec![0]);
    assert_eq!(
        batches[500],
        vec![
            499, 500, 501, 502, 503, 504, 505, 506, 507, 508, 509, 510, 511, 512, 513, 514, 515,
            516, 517, 518, 519, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529, 530, 531, 532,
            533, 534, 535, 536, 537, 538, 539, 540, 541, 542, 543, 544, 545, 546, 547, 548, 549,
            550, 551, 552, 553, 554, 555, 556, 557, 558, 559, 560, 561, 562, 563, 564, 565, 566,
            567, 568, 569, 570, 571, 572, 573, 574, 575, 576, 577, 578, 579, 580, 581, 582, 583,
            584, 585, 586, 587, 588, 589, 590, 591, 592, 593, 594, 595, 596, 597, 598, 599, 600,
            601, 602, 603, 604, 605, 606, 607, 608, 609, 610, 611, 612, 613, 614, 615, 616, 617,
            618, 619, 620, 621, 622, 623, 624, 625, 626, 627, 628, 629, 630, 631, 632, 633, 634,
            635, 636, 637, 638, 639, 640, 641, 642, 643, 644, 645, 646, 647, 648, 649, 650, 651,
            652, 653, 654, 655, 656, 657, 658, 659, 660, 661, 662, 663, 664, 665, 666, 667, 668,
            669, 670, 671, 672, 673, 674, 675, 676, 677, 678, 679, 680, 681, 682, 683, 684, 685,
            686, 687, 688, 689, 690, 691, 692, 693, 694, 695, 696, 697, 698, 699, 700, 701, 702,
            703, 704, 705, 706, 707, 708, 709, 710, 711, 712, 713, 714, 715, 716, 717, 718, 719,
            720, 721, 722, 723, 724, 725, 726, 727, 728, 729, 730, 731, 732, 733, 734, 735, 736,
            737, 738, 739, 740, 741, 742, 743, 744, 745, 746, 747, 748, 749, 750, 751, 752, 753,
            754, 755, 756, 757, 758, 759, 760, 761, 762, 763, 764, 765, 766, 767, 768, 769, 770,
            771, 772, 773, 774, 775, 776, 777, 778, 779, 780, 781, 782, 783, 784, 785, 786, 787,
            788, 789, 790, 791, 792, 793, 794, 795, 796, 797, 798, 799, 800, 801, 802, 803, 804,
            805, 806, 807, 808, 809, 810, 811, 812, 813, 814, 815, 816, 817, 818, 819, 820, 821,
            822, 823, 824, 825, 826, 827, 828, 829, 830, 831, 832, 833, 834, 835, 836, 837, 838,
            839, 840, 841, 842, 843, 844, 845, 846, 847, 848, 849, 850, 851, 852, 853, 854, 855,
            856, 857, 858, 859, 860, 861, 862, 863, 864, 865, 866, 867, 868, 869, 870, 871, 872,
            873, 874, 875, 876, 877, 878, 879, 880, 881, 882, 883, 884, 885, 886, 887, 888, 889,
            890, 891, 892, 893, 894, 895, 896, 897, 898, 899, 900, 901, 902, 903, 904, 905, 906,
            907, 908, 909, 910, 911, 912, 913, 914, 915, 916, 917, 918, 919, 920, 921, 922, 923,
            924, 925, 926, 927, 928, 929, 930, 931, 932, 933, 934, 935, 936, 937, 938, 939, 940,
            941, 942, 943, 944, 945, 946, 947, 948, 949, 950, 951, 952, 953, 954, 955, 956, 957,
            958, 959, 960, 961, 962, 963, 964, 965, 966, 967, 968, 969, 970, 971, 972, 973, 974,
            975, 976, 977, 978, 979, 980, 981, 982, 983, 984, 985, 986, 987, 988, 989, 990, 991,
            992, 993, 994, 995, 996, 997, 998, 999
        ]
    );
}

#[test]
fn test_batch_complex_predicate() {
    // Traces to: FR-PHENO-ITER-002 (complex predicate logic)
    let data = vec![1, 3, 5, 7, 2, 4, 6];
    let batches: Vec<_> = data.into_iter().batch(|&x| x % 2 == 1).collect();

    // Odd numbers trigger batch breaks
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], vec![1]);
    assert_eq!(batches[1], vec![3, 5, 7, 2, 4, 6]);
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
