//! Module repository -- CRUD operations for the `modules` table and `module_feature_tags`.
//!
//! Traces to: FR-M01, FR-M02, FR-M04, FR-M07

mod crud;
mod mappers;
mod tags;

pub use crud::{
    create_module, delete_module, get_module, get_module_by_slug, get_module_with_features,
    list_child_modules, list_root_modules, update_module, would_create_circular_ref,
};
pub use tags::{tag_feature_to_module, untag_feature_from_module};
