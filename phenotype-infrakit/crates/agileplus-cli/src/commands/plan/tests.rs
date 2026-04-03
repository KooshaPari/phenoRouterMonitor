use super::parsing::FunctionalRequirement;
use super::*;

#[test]
fn parse_frs_basic() {
    let spec = "## Functional Requirements\n- **FR-001**: Login must work\n- **FR-002**: Logout must work\n";
    let frs = parse_functional_requirements(spec);
    assert_eq!(frs.len(), 2);
    assert_eq!(frs[0].id, "FR-001");
    assert!(frs[0].description.contains("Login"));
}

#[test]
fn parse_frs_empty() {
    let frs = parse_functional_requirements("No FRs here");
    assert_eq!(frs.len(), 0);
}

#[test]
fn group_frs_small() {
    let frs: Vec<FunctionalRequirement> = (1..=5)
        .map(|i| FunctionalRequirement {
            id: format!("FR-{i:03}"),
            description: format!("desc {i}"),
        })
        .collect();
    let groups = group_frs_into_wps(&frs, 20);
    let total: usize = groups.iter().map(|g| g.len()).sum();
    assert_eq!(total, 5);
}

#[test]
fn group_frs_empty() {
    assert!(group_frs_into_wps(&[], 10).is_empty());
}

#[test]
fn slugify_basic() {
    assert_eq!(slugify("Hello World WP01"), "hello-world-wp01");
}

#[test]
fn generate_plan_md_contains_sections() {
    let wps = vec![WorkPackage::new(
        1,
        "Auth Module (WP01)",
        1,
        "- FR-001 login",
    )];
    let plan = generate_plan_md("my-feature", &wps, &[], &[]);
    assert!(plan.contains("# Plan: my-feature"));
    assert!(plan.contains("WP01: Auth Module"));
    assert!(plan.contains("Execution Waves"));
}
