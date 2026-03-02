//! T046: State Mapping — bidirectional Plane.so state group ↔ AgilePlus FeatureState.
//!
//! Traceability: WP08-T046

use std::collections::HashMap;

use agileplus_domain::domain::state_machine::FeatureState;

/// Plane.so state group names.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlaneStateGroup {
    Backlog,
    Unstarted,
    Started,
    Completed,
    Cancelled,
    Unknown(String),
}

impl PlaneStateGroup {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "backlog" => Self::Backlog,
            "unstarted" | "todo" => Self::Unstarted,
            "started" | "in_progress" | "in progress" => Self::Started,
            "completed" | "done" => Self::Completed,
            "cancelled" | "canceled" => Self::Cancelled,
            other => Self::Unknown(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Backlog => "backlog",
            Self::Unstarted => "unstarted",
            Self::Started => "started",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

/// A custom state override in the mapper config.
#[derive(Debug, Clone)]
pub struct StateOverride {
    /// Plane state group (e.g. "started").
    pub plane_group: String,
    /// Optional specific state name for finer matching.
    pub plane_name: Option<String>,
    /// Target AgilePlus FeatureState.
    pub feature_state: FeatureState,
}

/// Configuration for PlaneStateMapper.
#[derive(Debug, Default)]
pub struct PlaneStateMapperConfig {
    /// Custom per-state overrides; if present these take precedence over defaults.
    pub overrides: Vec<StateOverride>,
    /// Map from FeatureState → Plane state UUID for outbound sync.
    pub state_id_map: HashMap<FeatureState, (String, String)>,
}

/// Bidirectional mapper between Plane.so state groups and AgilePlus FeatureState.
///
/// Default mapping:
/// - Backlog    → Created
/// - Unstarted  → Specified
/// - Started    → Implementing
/// - Completed  → Validated
/// - Cancelled  → (warning, returns Validated as closest terminal)
#[derive(Debug)]
pub struct PlaneStateMapper {
    config: PlaneStateMapperConfig,
}

impl PlaneStateMapper {
    /// Create a mapper with default mappings.
    pub fn new() -> Self {
        Self {
            config: PlaneStateMapperConfig::default(),
        }
    }

    /// Create a mapper with custom config.
    pub fn with_config(config: PlaneStateMapperConfig) -> Self {
        Self { config }
    }

    /// Map from a Plane.so state group + state name → AgilePlus FeatureState.
    ///
    /// Custom overrides are checked first (most specific first: group+name, then group-only).
    /// Falls back to the default group mapping if no override matches.
    pub fn from_plane(&self, state_group: &str, state_name: &str) -> FeatureState {
        // Check overrides: group + name match first.
        for o in &self.config.overrides {
            if o.plane_group.eq_ignore_ascii_case(state_group) {
                if let Some(ref name) = o.plane_name {
                    if name.eq_ignore_ascii_case(state_name) {
                        return o.feature_state;
                    }
                }
            }
        }
        // Check overrides: group-only match.
        for o in &self.config.overrides {
            if o.plane_group.eq_ignore_ascii_case(state_group) && o.plane_name.is_none() {
                return o.feature_state;
            }
        }

        // Default mapping by group.
        match PlaneStateGroup::from_str(state_group) {
            PlaneStateGroup::Backlog => FeatureState::Created,
            PlaneStateGroup::Unstarted => FeatureState::Specified,
            PlaneStateGroup::Started => FeatureState::Implementing,
            PlaneStateGroup::Completed => FeatureState::Validated,
            PlaneStateGroup::Cancelled => {
                tracing::warn!(
                    state_group = state_group,
                    state_name = state_name,
                    "Plane.so Cancelled state has no AgilePlus equivalent; defaulting to Validated"
                );
                FeatureState::Validated
            }
            PlaneStateGroup::Unknown(ref g) => {
                tracing::warn!(
                    plane_group = g,
                    state_name = state_name,
                    "Unknown Plane.so state group; defaulting to Created"
                );
                FeatureState::Created
            }
        }
    }

    /// Map from an AgilePlus FeatureState → (plane_state_group, plane_state_id).
    ///
    /// Returns the configured state ID if present, otherwise returns a default
    /// sentinel group with an empty state ID.
    pub fn to_plane(&self, state: FeatureState) -> (String, String) {
        if let Some((group, id)) = self.config.state_id_map.get(&state) {
            return (group.clone(), id.clone());
        }
        // Return default group with empty ID (caller should handle missing ID).
        let group = match state {
            FeatureState::Created => "backlog",
            FeatureState::Specified | FeatureState::Researched | FeatureState::Planned => {
                "unstarted"
            }
            FeatureState::Implementing => "started",
            FeatureState::Validated | FeatureState::Shipped | FeatureState::Retrospected => {
                "completed"
            }
        };
        (group.to_string(), String::new())
    }
}

impl Default for PlaneStateMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_backlog_maps_to_created() {
        let mapper = PlaneStateMapper::new();
        assert_eq!(mapper.from_plane("backlog", "Backlog"), FeatureState::Created);
    }

    #[test]
    fn default_unstarted_maps_to_specified() {
        let mapper = PlaneStateMapper::new();
        assert_eq!(
            mapper.from_plane("unstarted", "Todo"),
            FeatureState::Specified
        );
    }

    #[test]
    fn default_started_maps_to_implementing() {
        let mapper = PlaneStateMapper::new();
        assert_eq!(
            mapper.from_plane("started", "In Progress"),
            FeatureState::Implementing
        );
    }

    #[test]
    fn default_completed_maps_to_validated() {
        let mapper = PlaneStateMapper::new();
        assert_eq!(
            mapper.from_plane("completed", "Done"),
            FeatureState::Validated
        );
    }

    #[test]
    fn cancelled_warns_and_defaults() {
        let mapper = PlaneStateMapper::new();
        // Cancelled has no direct equivalent; returns Validated with warning.
        let result = mapper.from_plane("cancelled", "Wont Fix");
        assert_eq!(result, FeatureState::Validated);
    }

    #[test]
    fn to_plane_created_returns_backlog() {
        let mapper = PlaneStateMapper::new();
        let (group, _id) = mapper.to_plane(FeatureState::Created);
        assert_eq!(group, "backlog");
    }

    #[test]
    fn to_plane_implementing_returns_started() {
        let mapper = PlaneStateMapper::new();
        let (group, _id) = mapper.to_plane(FeatureState::Implementing);
        assert_eq!(group, "started");
    }

    #[test]
    fn to_plane_with_config_returns_custom_id() {
        let mut config = PlaneStateMapperConfig::default();
        config
            .state_id_map
            .insert(FeatureState::Implementing, ("started".into(), "uuid-123".into()));
        let mapper = PlaneStateMapper::with_config(config);
        let (group, id) = mapper.to_plane(FeatureState::Implementing);
        assert_eq!(group, "started");
        assert_eq!(id, "uuid-123");
    }

    #[test]
    fn override_takes_precedence() {
        let config = PlaneStateMapperConfig {
            overrides: vec![StateOverride {
                plane_group: "started".into(),
                plane_name: Some("review".into()),
                feature_state: FeatureState::Validated,
            }],
            state_id_map: HashMap::new(),
        };
        let mapper = PlaneStateMapper::with_config(config);
        // Override matches: started + "review" → Validated
        assert_eq!(
            mapper.from_plane("started", "review"),
            FeatureState::Validated
        );
        // No override: started + "coding" → Implementing (default)
        assert_eq!(
            mapper.from_plane("started", "coding"),
            FeatureState::Implementing
        );
    }
}
