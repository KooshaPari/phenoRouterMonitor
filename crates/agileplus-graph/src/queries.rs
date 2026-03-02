use crate::store::{GraphError, GraphStore};
use serde_json::json;

pub struct GraphQueries<'a> {
    store: &'a GraphStore,
}

impl<'a> GraphQueries<'a> {
    pub fn new(store: &'a GraphStore) -> Self {
        GraphQueries { store }
    }

    /// Get all transitive dependencies of a feature.
    pub async fn get_dependency_chain(
        &self,
        feature_id: i64,
    ) -> Result<Vec<(i64, String)>, GraphError> {
        let rows = self
            .store
            .query_cypher(
                "MATCH (f:Feature {id: $id})-[:DEPENDS_ON*]->(dep:Feature) RETURN dep.id AS id, dep.friendly_name AS name",
                &json!({"id": feature_id}),
            )
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let id = row.get("id")?.as_i64()?;
                let name = row.get("name")?.as_str()?.to_string();
                Some((id, name))
            })
            .collect())
    }

    /// Get what blocks a work package (transitive).
    pub async fn get_blocking_path(
        &self,
        wp_id: i64,
    ) -> Result<Vec<(i64, String)>, GraphError> {
        let rows = self
            .store
            .query_cypher(
                "MATCH (blocker:WorkPackage)-[:BLOCKS*]->(w:WorkPackage {id: $id}) RETURN blocker.id AS id, blocker.title AS title",
                &json!({"id": wp_id}),
            )
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let id = row.get("id")?.as_i64()?;
                let title = row.get("title")?.as_str()?.to_string();
                Some((id, title))
            })
            .collect())
    }

    /// Get all features in a project.
    pub async fn get_project_features(
        &self,
        project_slug: &str,
    ) -> Result<Vec<(i64, String)>, GraphError> {
        let rows = self
            .store
            .query_cypher(
                "MATCH (f:Feature)-[:IN_PROJECT]->(p:Project {slug: $slug}) RETURN f.id AS id, f.friendly_name AS name ORDER BY f.id",
                &json!({"slug": project_slug}),
            )
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let id = row.get("id")?.as_i64()?;
                let name = row.get("name")?.as_str()?.to_string();
                Some((id, name))
            })
            .collect())
    }

    /// Get all assigned work packages for an agent.
    pub async fn get_agent_workload(
        &self,
        agent_name: &str,
    ) -> Result<Vec<(i64, String)>, GraphError> {
        let rows = self
            .store
            .query_cypher(
                "MATCH (w:WorkPackage)-[:ASSIGNED_TO]->(a:Agent {name: $name}) RETURN w.id AS id, w.title AS title ORDER BY w.ordinal",
                &json!({"name": agent_name}),
            )
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let id = row.get("id")?.as_i64()?;
                let title = row.get("title")?.as_str()?.to_string();
                Some((id, title))
            })
            .collect())
    }

    /// Get features by label.
    pub async fn get_features_by_label(
        &self,
        label_name: &str,
    ) -> Result<Vec<(i64, String)>, GraphError> {
        let rows = self
            .store
            .query_cypher(
                "MATCH (f:Feature)-[:TAGGED]->(l:Label {name: $label}) RETURN f.id AS id, f.friendly_name AS name ORDER BY f.id",
                &json!({"label": label_name}),
            )
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let id = row.get("id")?.as_i64()?;
                let name = row.get("name")?.as_str()?.to_string();
                Some((id, name))
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GraphConfig;

    #[tokio::test]
    async fn test_empty_dependency_chain() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let queries = GraphQueries::new(&store);

        let chain = queries.get_dependency_chain(1).await.unwrap();
        assert!(chain.is_empty());
    }

    #[tokio::test]
    async fn test_empty_blocking_path() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let queries = GraphQueries::new(&store);

        let blockers = queries.get_blocking_path(1).await.unwrap();
        assert!(blockers.is_empty());
    }

    #[tokio::test]
    async fn test_empty_project_features() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let queries = GraphQueries::new(&store);

        let features = queries.get_project_features("nonexistent").await.unwrap();
        assert!(features.is_empty());
    }

    #[tokio::test]
    async fn test_empty_agent_workload() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let queries = GraphQueries::new(&store);

        let workload = queries.get_agent_workload("nobody").await.unwrap();
        assert!(workload.is_empty());
    }

    #[tokio::test]
    async fn test_empty_features_by_label() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let queries = GraphQueries::new(&store);

        let features = queries.get_features_by_label("nolabel").await.unwrap();
        assert!(features.is_empty());
    }
}
