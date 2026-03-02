use crate::store::{GraphError, GraphStore};
use serde_json::json;

pub struct RelationshipStore<'a> {
    store: &'a GraphStore,
}

impl<'a> RelationshipStore<'a> {
    pub fn new(store: &'a GraphStore) -> Self {
        RelationshipStore { store }
    }

    /// Feature -[:OWNS]-> WorkPackage
    pub async fn owns(&self, feature_id: i64, wp_id: i64) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "MATCH (f:Feature {id: $f_id}), (w:WorkPackage {id: $w_id}) CREATE (f)-[:OWNS]->(w)",
                &json!({"f_id": feature_id, "w_id": wp_id}),
            )
            .await
    }

    /// WorkPackage -[:ASSIGNED_TO]-> Agent
    pub async fn assign_to_agent(
        &self,
        wp_id: i64,
        agent_name: String,
    ) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "MATCH (w:WorkPackage {id: $wp_id}), (a:Agent {name: $agent_name}) CREATE (w)-[:ASSIGNED_TO]->(a)",
                &json!({"wp_id": wp_id, "agent_name": agent_name}),
            )
            .await
    }

    /// Feature -[:DEPENDS_ON]-> Feature
    pub async fn depends_on(
        &self,
        from_feature_id: i64,
        to_feature_id: i64,
    ) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "MATCH (f1:Feature {id: $f1_id}), (f2:Feature {id: $f2_id}) CREATE (f1)-[:DEPENDS_ON]->(f2)",
                &json!({"f1_id": from_feature_id, "f2_id": to_feature_id}),
            )
            .await
    }

    /// WorkPackage -[:BLOCKS]-> WorkPackage
    pub async fn blocks(
        &self,
        blocking_wp_id: i64,
        blocked_wp_id: i64,
    ) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "MATCH (w1:WorkPackage {id: $w1_id}), (w2:WorkPackage {id: $w2_id}) CREATE (w1)-[:BLOCKS]->(w2)",
                &json!({"w1_id": blocking_wp_id, "w2_id": blocked_wp_id}),
            )
            .await
    }

    /// Feature -[:TAGGED]-> Label
    pub async fn tag_feature(
        &self,
        feature_id: i64,
        label_name: String,
    ) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "MATCH (f:Feature {id: $f_id}), (l:Label {name: $label_name}) CREATE (f)-[:TAGGED]->(l)",
                &json!({"f_id": feature_id, "label_name": label_name}),
            )
            .await
    }

    /// Feature -[:IN_PROJECT]-> Project
    pub async fn feature_in_project(
        &self,
        feature_id: i64,
        project_slug: String,
    ) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "MATCH (f:Feature {id: $f_id}), (p:Project {slug: $p_slug}) CREATE (f)-[:IN_PROJECT]->(p)",
                &json!({"f_id": feature_id, "p_slug": project_slug}),
            )
            .await
    }

    /// WorkPackage -[:IN_PROJECT]-> Project
    pub async fn workpackage_in_project(
        &self,
        wp_id: i64,
        project_slug: String,
    ) -> Result<(), GraphError> {
        self.store
            .run_cypher(
                "MATCH (w:WorkPackage {id: $w_id}), (p:Project {slug: $p_slug}) CREATE (w)-[:IN_PROJECT]->(p)",
                &json!({"w_id": wp_id, "p_slug": project_slug}),
            )
            .await
    }

    /// Delete a relationship between two nodes.
    pub async fn delete_relationship(
        &self,
        from_type: &str,
        from_id: i64,
        rel_type: &str,
        to_type: &str,
        to_id: i64,
    ) -> Result<(), GraphError> {
        let query = format!(
            "MATCH (from:{from_type} {{id: $from_id}})-[r:{rel_type}]->(to:{to_type} {{id: $to_id}}) DELETE r"
        );
        self.store
            .run_cypher(&query, &json!({"from_id": from_id, "to_id": to_id}))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GraphConfig;
    use crate::nodes::NodeStore;

    #[tokio::test]
    async fn test_create_owns_relationship() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let nodes = NodeStore::new(&store);
        let rels = RelationshipStore::new(&store);

        nodes
            .create_feature(1, "f1".into(), "open".into(), "Feature 1".into())
            .await
            .unwrap();
        nodes
            .create_workpackage(10, "WP10".into(), "todo".into(), 1)
            .await
            .unwrap();

        assert!(rels.owns(1, 10).await.is_ok());
    }

    #[tokio::test]
    async fn test_create_depends_on_relationship() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let nodes = NodeStore::new(&store);
        let rels = RelationshipStore::new(&store);

        nodes
            .create_feature(1, "f1".into(), "open".into(), "F1".into())
            .await
            .unwrap();
        nodes
            .create_feature(2, "f2".into(), "open".into(), "F2".into())
            .await
            .unwrap();

        assert!(rels.depends_on(1, 2).await.is_ok());
    }

    #[tokio::test]
    async fn test_assign_to_agent() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let nodes = NodeStore::new(&store);
        let rels = RelationshipStore::new(&store);

        nodes
            .create_workpackage(10, "WP10".into(), "todo".into(), 1)
            .await
            .unwrap();
        nodes
            .create_agent("claude".into(), "ai".into())
            .await
            .unwrap();

        assert!(rels.assign_to_agent(10, "claude".into()).await.is_ok());
    }

    #[tokio::test]
    async fn test_delete_relationship() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let rels = RelationshipStore::new(&store);

        assert!(rels
            .delete_relationship("Feature", 1, "DEPENDS_ON", "Feature", 2)
            .await
            .is_ok());
    }
}
