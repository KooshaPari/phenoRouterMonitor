// Integration test for seed module
use agileplus_dashboard::seed::seed_dogfood_features;
use agileplus_domain::domain::state_machine::FeatureState;

#[test]
fn seed_creates_all_features() {
    let (features, _work_packages) = seed_dogfood_features();
    assert_eq!(
        features.len(),
        37,
        "Should create 4 AgilePlus + 33 SpecKitty features"
    );
}

#[test]
fn seed_assigns_correct_ids() {
    let (features, _work_packages) = seed_dogfood_features();
    assert_eq!(features[0].id, 1);
    assert_eq!(features[1].id, 2);
    assert_eq!(features[2].id, 3);
    assert_eq!(features[3].id, 4);
    // SpecKitty features start at ID 5
    assert_eq!(features[4].id, 5);
    assert_eq!(features[features.len() - 1].id, 37);
}

#[test]
fn seed_assigns_correct_states() {
    let (features, _work_packages) = seed_dogfood_features();
    // AgilePlus features 1-3: Shipped, feature 4: Implementing
    assert_eq!(features[0].state, FeatureState::Shipped);
    assert_eq!(features[1].state, FeatureState::Shipped);
    assert_eq!(features[2].state, FeatureState::Shipped);
    assert_eq!(features[3].state, FeatureState::Implementing);
    // All SpecKitty features: Shipped
    for f in &features[4..] {
        assert_eq!(
            f.state,
            FeatureState::Shipped,
            "SpecKitty feature {} should be Shipped",
            f.slug
        );
    }
}

#[test]
fn seed_creates_work_packages_for_all_features() {
    let (features, work_packages) = seed_dogfood_features();
    assert_eq!(
        work_packages.len(),
        features.len(),
        "Should have work packages for all features"
    );
    for f in &features {
        assert!(
            work_packages.contains_key(&f.id),
            "Missing WPs for feature {}",
            f.id
        );
    }
}

#[test]
fn seed_creates_correct_agileplus_work_package_counts() {
    let (_features, work_packages) = seed_dogfood_features();
    assert_eq!(work_packages.get(&1).map(|v| v.len()), Some(4));
    assert_eq!(work_packages.get(&2).map(|v| v.len()), Some(3));
    assert_eq!(work_packages.get(&3).map(|v| v.len()), Some(4));
    assert_eq!(work_packages.get(&4).map(|v| v.len()), Some(3));
}

#[test]
fn seed_feature_has_labels() {
    let (features, _work_packages) = seed_dogfood_features();
    for f in &features {
        assert!(
            !f.labels.is_empty(),
            "Feature {} should have labels",
            f.slug
        );
    }
}

#[test]
fn seed_speckitty_features_tagged() {
    let (features, _work_packages) = seed_dogfood_features();
    for f in &features[4..] {
        assert!(
            f.labels.contains(&"specKitty".to_string()),
            "SpecKitty feature {} missing specKitty label",
            f.slug
        );
    }
}

#[test]
fn seed_creates_seeded_dashboard_store() {
    use agileplus_dashboard::app_state::DashboardStore;

    let store = DashboardStore::seeded();
    assert_eq!(store.features.len(), 37);
    assert!(!store.health.is_empty());
    assert!(store.work_packages.contains_key(&1));
    assert!(store.work_packages.contains_key(&37));
}
