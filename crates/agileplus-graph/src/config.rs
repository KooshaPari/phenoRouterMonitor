#[derive(Clone, Debug)]
pub struct GraphConfig {
    pub bolt_uri: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl GraphConfig {
    pub fn new(bolt_uri: String, username: String, password: String) -> Self {
        GraphConfig {
            bolt_uri,
            username,
            password,
            database: "neo4j".to_string(),
        }
    }

    pub fn with_database(mut self, db: String) -> Self {
        self.database = db;
        self
    }
}

impl Default for GraphConfig {
    fn default() -> Self {
        GraphConfig {
            bolt_uri: "bolt://localhost:7687".to_string(),
            username: "neo4j".to_string(),
            password: "agileplus".to_string(),
            database: "neo4j".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GraphConfig::default();
        assert_eq!(config.bolt_uri, "bolt://localhost:7687");
        assert_eq!(config.username, "neo4j");
        assert_eq!(config.database, "neo4j");
    }

    #[test]
    fn test_config_with_database() {
        let config = GraphConfig::new(
            "bolt://localhost:7687".to_string(),
            "user".to_string(),
            "pass".to_string(),
        )
        .with_database("mydb".to_string());
        assert_eq!(config.database, "mydb");
    }
}
