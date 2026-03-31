use crate::store::{GraphError, GraphStore};
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphHealth {
    Healthy,
    Unavailable,
}

pub struct GraphHealthChecker<'a> {
    store: &'a GraphStore,
}

impl<'a> GraphHealthChecker<'a> {
    pub fn new(store: &'a GraphStore) -> Self {
        GraphHealthChecker { store }
    }

    pub async fn check(&self) -> GraphHealth {
        match self.store.health_check().await {
            Ok(()) => GraphHealth::Healthy,
            Err(_) => GraphHealth::Unavailable,
        }
    }
}

pub struct IndexManager<'a> {
    store: &'a GraphStore,
}

impl<'a> IndexManager<'a> {
    pub fn new(store: &'a GraphStore) -> Self {
        IndexManager { store }
    }

    pub async fn create_indexes(&self) -> Result<(), GraphError> {
        let indexes = [
            "CREATE INDEX feature_slug IF NOT EXISTS FOR (f:Feature) ON (f.slug)",
            "CREATE INDEX workpackage_title IF NOT EXISTS FOR (w:WorkPackage) ON (w.title)",
            "CREATE INDEX agent_type IF NOT EXISTS FOR (a:Agent) ON (a.type)",
            "CREATE INDEX label_color IF NOT EXISTS FOR (l:Label) ON (l.color)",
        ];

        for cypher in &indexes {
            let _ = self.store.run_cypher(cypher, &json!({})).await;
        }

        Ok(())
    }

    pub async fn delete_indexes(&self) -> Result<(), GraphError> {
        let drops = [
            "DROP INDEX feature_slug IF EXISTS",
            "DROP INDEX workpackage_title IF EXISTS",
        ];

        for cypher in &drops {
            let _ = self.store.run_cypher(cypher, &json!({})).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GraphConfig;

    #[tokio::test]
    async fn test_health_check_healthy() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let checker = GraphHealthChecker::new(&store);
        assert_eq!(checker.check().await, GraphHealth::Healthy);
    }

    #[tokio::test]
    async fn test_create_indexes() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let mgr = IndexManager::new(&store);
        assert!(mgr.create_indexes().await.is_ok());
    }

    #[tokio::test]
    async fn test_delete_indexes() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let mgr = IndexManager::new(&store);
        assert!(mgr.delete_indexes().await.is_ok());
    }
}
