use super::worker::slugify;

#[test]
fn slugify_basic() {
    assert_eq!(
        slugify("Implement Auth Module (WP01)"),
        "implement-auth-module-wp01"
    );
}
