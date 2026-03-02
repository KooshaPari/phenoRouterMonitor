use crate::store::{GraphError, GraphStore};
use serde_json::json;

pub struct NodeStore<'a> {
    store: &'a GraphStore,
}

impl<'a> NodeStore<'a> {
    pub fn new(store: &'a GraphStore) -> Self {
        NodeStore { store }
    }

    // === Feature Node Operations ===

    pub async fn create_feature(
        &self,
        id: i64,
        slug: String,
        state: String,
        friendly_name: String,
    ) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "CREATE (f:Feature {id: $id, slug: $slug, state: $state, friendly_name: $friendly_name})",
                &json!({"id": id, "slug": slug, "state": state, "friendly_name": friendly_name}),
            )
            .await
    }

    pub async fn get_feature(
        &self,
        feature_id: i64,
    ) -> Result<Option<serde_json::Value>, GraphError> {
        let rows = self
            .store
            .query_cypher(
                "MATCH (f:Feature {id: $id}) RETURN f",
                &json!({"id": feature_id}),
            )
            .await?;
        Ok(rows.into_iter().next())
    }

    pub async fn update_feature_state(
        &self,
        feature_id: i64,
        new_state: String,
    ) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "MATCH (f:Feature {id: $id}) SET f.state = $state",
                &json!({"id": feature_id, "state": new_state}),
            )
            .await
    }

    pub async fn delete_feature(&self, feature_id: i64) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "MATCH (f:Feature {id: $id}) DETACH DELETE f",
                &json!({"id": feature_id}),
            )
            .await
    }

    // === WorkPackage Node Operations ===

    pub async fn create_workpackage(
        &self,
        id: i64,
        title: String,
        state: String,
        ordinal: i32,
    ) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "CREATE (w:WorkPackage {id: $id, title: $title, state: $state, ordinal: $ordinal})",
                &json!({"id": id, "title": title, "state": state, "ordinal": ordinal}),
            )
            .await
    }

    pub async fn get_workpackage(
        &self,
        wp_id: i64,
    ) -> Result<Option<serde_json::Value>, GraphError> {
        let rows = self
            .store
            .query_cypher(
                "MATCH (w:WorkPackage {id: $id}) RETURN w",
                &json!({"id": wp_id}),
            )
            .await?;
        Ok(rows.into_iter().next())
    }

    pub async fn delete_workpackage(&self, wp_id: i64) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "MATCH (w:WorkPackage {id: $id}) DETACH DELETE w",
                &json!({"id": wp_id}),
            )
            .await
    }

    // === Agent Node Operations ===

    pub async fn create_agent(&self, name: String, agent_type: String) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "CREATE (a:Agent {name: $name, type: $type})",
                &json!({"name": name, "type": agent_type}),
            )
            .await
    }

    pub async fn get_agent(&self, name: &str) -> Result<Option<serde_json::Value>, GraphError> {
        let rows = self
            .store
            .query_cypher(
                "MATCH (a:Agent {name: $name}) RETURN a",
                &json!({"name": name}),
            )
            .await?;
        Ok(rows.into_iter().next())
    }

    // === Label Node Operations ===

    pub async fn create_label(&self, name: String, color: String) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "CREATE (l:Label {name: $name, color: $color})",
                &json!({"name": name, "color": color}),
            )
            .await
    }

    // === Project Node Operations ===

    pub async fn create_project(&self, name: String, slug: String) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "CREATE (p:Project {name: $name, slug: $slug})",
                &json!({"name": name, "slug": slug}),
            )
            .await
    }

    pub async fn get_project(
        &self,
        slug: &str,
    ) -> Result<Option<serde_json::Value>, GraphError> {
        let rows = self
            .store
            .query_cypher(
                "MATCH (p:Project {slug: $slug}) RETURN p",
                &json!({"slug": slug}),
            )
            .await?;
        Ok(rows.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GraphConfig;

    #[tokio::test]
    async fn test_create_and_get_feature() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let nodes = NodeStore::new(&store);

        nodes
            .create_feature(1, "feat-1".into(), "open".into(), "Feature One".into())
            .await
            .unwrap();

        let result = nodes.get_feature(1).await.unwrap();
        assert!(result.is_some());
        let val = result.unwrap();
        assert_eq!(val["slug"], "feat-1");
    }

    #[tokio::test]
    async fn test_update_feature_state() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let nodes = NodeStore::new(&store);

        nodes
            .create_feature(2, "feat-2".into(), "open".into(), "Feature Two".into())
            .await
            .unwrap();

        nodes
            .update_feature_state(2, "in_progress".into())
            .await
            .unwrap();

        let result = nodes.get_feature(2).await.unwrap().unwrap();
        assert_eq!(result["state"], "in_progress");
    }

    #[tokio::test]
    async fn test_delete_feature() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let nodes = NodeStore::new(&store);

        nodes
            .create_feature(3, "feat-3".into(), "open".into(), "Feature Three".into())
            .await
            .unwrap();

        nodes.delete_feature(3).await.unwrap();

        let result = nodes.get_feature(3).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_and_get_workpackage() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let nodes = NodeStore::new(&store);

        nodes
            .create_workpackage(10, "WP-10".into(), "todo".into(), 1)
            .await
            .unwrap();

        let result = nodes.get_workpackage(10).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_create_and_get_agent() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let nodes = NodeStore::new(&store);

        nodes
            .create_agent("claude".into(), "ai".into())
            .await
            .unwrap();

        let result = nodes.get_agent("claude").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap()["type"], "ai");
    }

    #[tokio::test]
    async fn test_create_and_get_project() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let nodes = NodeStore::new(&store);

        nodes
            .create_project("AgilePlus".into(), "agileplus".into())
            .await
            .unwrap();

        let result = nodes.get_project("agileplus").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap()["name"], "AgilePlus");
    }

    #[tokio::test]
    async fn test_get_nonexistent_feature() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let nodes = NodeStore::new(&store);

        let result = nodes.get_feature(999).await.unwrap();
        assert!(result.is_none());
    }
}
