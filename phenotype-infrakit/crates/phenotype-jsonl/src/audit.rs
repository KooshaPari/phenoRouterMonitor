use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEntry {
    pub timestamp: String,
    pub event_type: String,
    pub agent_id: String,
    pub details: Value,
}

pub struct AuditLogger {
    file_path: String,
    file: File,
    buffer: Vec<AuditEntry>,
}

impl AuditLogger {
    pub fn new(file_path: &str) -> Result<Self, AuditError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .map_err(|e| AuditError::IoError(e.to_string()))?;

        Ok(Self {
            file_path: file_path.to_string(),
            file,
            buffer: Vec::new(),
        })
    }

    pub fn append(&mut self, entry: &AuditEntry) -> Result<(), AuditError> {
        self.buffer.push(entry.clone());
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), AuditError> {
        for entry in &self.buffer {
            let json =
                serde_json::to_string(entry).map_err(|e| AuditError::SerError(e.to_string()))?;
            writeln!(self.file, "{}", json).map_err(|e| AuditError::IoError(e.to_string()))?;
        }
        self.file
            .flush()
            .map_err(|e| AuditError::IoError(e.to_string()))?;
        self.buffer.clear();
        Ok(())
    }

    pub fn file_hash(&self) -> Result<String, AuditError> {
        let content =
            std::fs::read(&self.file_path).map_err(|e| AuditError::IoError(e.to_string()))?;
        let hash = blake3::hash(&content);
        Ok(hash.to_hex().to_string())
    }

    pub fn read_range(&self, start: usize, end: usize) -> Result<Vec<AuditEntry>, AuditError> {
        let file = File::open(&self.file_path).map_err(|e| AuditError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);

        let mut entries = Vec::new();
        for (idx, line) in reader.lines().enumerate() {
            if idx >= end {
                break;
            }
            if idx >= start {
                let line = line.map_err(|e| AuditError::IoError(e.to_string()))?;
                let entry: AuditEntry =
                    serde_json::from_str(&line).map_err(|e| AuditError::SerError(e.to_string()))?;
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    pub fn query(
        &self,
        event_type: Option<&str>,
        agent_id: Option<&str>,
    ) -> Result<Vec<AuditEntry>, AuditError> {
        let file = File::open(&self.file_path).map_err(|e| AuditError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);

        let mut results = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|e| AuditError::IoError(e.to_string()))?;
            let entry: AuditEntry =
                serde_json::from_str(&line).map_err(|e| AuditError::SerError(e.to_string()))?;

            let matches_type =
                event_type.is_none() || event_type == Some(entry.event_type.as_str());
            let matches_agent = agent_id.is_none() || agent_id == Some(entry.agent_id.as_str());

            if matches_type && matches_agent {
                results.push(entry);
            }
        }

        Ok(results)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Serialization error: {0}")]
    SerError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @trace FR-AUD-001
    #[test]
    fn test_audit_logger_writes_jsonl() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.jsonl");

        let mut logger = AuditLogger::new(log_path.to_str().unwrap()).unwrap();

        let entry = AuditEntry {
            timestamp: "2026-02-22T00:00:00Z".to_string(),
            event_type: "policy_check".to_string(),
            agent_id: "agent-1".to_string(),
            details: serde_json::json!({"rule_id": "FR-GOV-001", "passed": true}),
        };

        logger.append(&entry).unwrap();
        logger.flush().unwrap();

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("policy_check"));
        assert!(content.contains("agent-1"));
    }

    /// @trace FR-AUD-002
    #[test]
    fn test_audit_logger_immutability() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.jsonl");

        let mut logger = AuditLogger::new(log_path.to_str().unwrap()).unwrap();

        let entry = AuditEntry {
            timestamp: "2026-02-22T00:00:00Z".to_string(),
            event_type: "test".to_string(),
            agent_id: "agent-1".to_string(),
            details: serde_json::json!({}),
        };

        logger.append(&entry).unwrap();
        logger.flush().unwrap();
        let hash1 = logger.file_hash().unwrap();

        logger.append(&entry).unwrap();
        logger.flush().unwrap();
        let hash2 = logger.file_hash().unwrap();

        assert_ne!(hash1, hash2);
    }

    /// @trace FR-AUD-003
    #[test]
    fn test_audit_logger_read_range() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.jsonl");

        let mut logger = AuditLogger::new(log_path.to_str().unwrap()).unwrap();

        for i in 0..5 {
            let entry = AuditEntry {
                timestamp: format!("2026-02-22T0{}:00:00Z", i),
                event_type: "event".to_string(),
                agent_id: format!("agent-{}", i),
                details: serde_json::json!({"index": i}),
            };
            logger.append(&entry).unwrap();
        }
        logger.flush().unwrap();

        let entries = logger.read_range(1, 3).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].agent_id, "agent-1");
        assert_eq!(entries[1].agent_id, "agent-2");
    }

    /// @trace FR-AUD-004
    #[test]
    fn test_audit_logger_query_by_event_type() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.jsonl");

        let mut logger = AuditLogger::new(log_path.to_str().unwrap()).unwrap();

        logger
            .append(&AuditEntry {
                timestamp: "2026-02-22T00:00:00Z".to_string(),
                event_type: "policy_check".to_string(),
                agent_id: "agent-1".to_string(),
                details: serde_json::json!({}),
            })
            .unwrap();
        logger
            .append(&AuditEntry {
                timestamp: "2026-02-22T01:00:00Z".to_string(),
                event_type: "cost_check".to_string(),
                agent_id: "agent-2".to_string(),
                details: serde_json::json!({}),
            })
            .unwrap();
        logger
            .append(&AuditEntry {
                timestamp: "2026-02-22T02:00:00Z".to_string(),
                event_type: "policy_check".to_string(),
                agent_id: "agent-3".to_string(),
                details: serde_json::json!({}),
            })
            .unwrap();
        logger.flush().unwrap();

        let results = logger.query(Some("policy_check"), None).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.event_type == "policy_check"));
    }

    /// @trace FR-AUD-004
    #[test]
    fn test_audit_logger_query_by_agent_id() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.jsonl");

        let mut logger = AuditLogger::new(log_path.to_str().unwrap()).unwrap();

        for i in 0..3 {
            logger
                .append(&AuditEntry {
                    timestamp: format!("2026-02-22T0{}:00:00Z", i),
                    event_type: "event".to_string(),
                    agent_id: format!("agent-{}", i % 2),
                    details: serde_json::json!({}),
                })
                .unwrap();
        }
        logger.flush().unwrap();

        let results = logger.query(None, Some("agent-0")).unwrap();
        assert_eq!(results.len(), 2);
    }
}
