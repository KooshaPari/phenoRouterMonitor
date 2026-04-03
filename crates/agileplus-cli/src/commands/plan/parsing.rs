#[derive(Debug, Clone)]
pub(crate) struct FunctionalRequirement {
    pub(crate) id: String,
    pub(crate) description: String,
}

/// Parse `FR-NNN: description` lines from spec content.
pub(crate) fn parse_functional_requirements(spec: &str) -> Vec<FunctionalRequirement> {
    let mut frs = Vec::new();
    for line in spec.lines() {
        if let Some(pos) = line.find("FR-") {
            let rest = &line[pos..];
            let id_end = rest[3..]
                .find(|c: char| !c.is_ascii_digit())
                .map(|p| p + 3)
                .unwrap_or(rest.len());
            let id = &rest[..id_end];
            let description = if let Some(colon) = rest.find(':') {
                rest[colon + 1..]
                    .trim()
                    .trim_matches('*')
                    .trim()
                    .to_string()
            } else {
                rest.to_string()
            };
            if !description.is_empty() && id.len() > 3 {
                frs.push(FunctionalRequirement {
                    id: id.to_string(),
                    description,
                });
            }
        }
    }
    frs.dedup_by_key(|fr| fr.id.clone());
    frs
}

/// Group FRs into logical WP batches (3-7 FRs per WP).
pub(crate) fn group_frs_into_wps(
    frs: &[FunctionalRequirement],
    max_wps: usize,
) -> Vec<Vec<FunctionalRequirement>> {
    if frs.is_empty() {
        return Vec::new();
    }
    let target_per_wp =
        ((frs.len() as f64) / (max_wps as f64).min(frs.len() as f64)).ceil() as usize;
    let per_wp = target_per_wp.clamp(3, 7);
    frs.chunks(per_wp).map(|chunk| chunk.to_vec()).collect()
}

/// Derive a human-readable WP title from a group of FRs.
pub(crate) fn derive_wp_title(frs: &[FunctionalRequirement], index: usize) -> String {
    if frs.is_empty() {
        return format!("Work Package {index:02}");
    }
    let hint = &frs[0].description;
    let truncated: String = hint.chars().take(50).collect();
    format!("{truncated} (WP{index:02})")
}
