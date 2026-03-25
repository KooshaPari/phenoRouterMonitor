#[derive(Debug, Default, Clone)]
pub struct ExportStats {
    pub events_exported: usize,
    pub snapshots_exported: usize,
    pub sync_mappings_exported: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct EntityRef {
    pub entity_type: String,
    pub entity_id: i64,
}
