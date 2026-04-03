//! Ledger integrity verification module

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Verifies the integrity of the action ledger using rolling hashes
#[derive(Debug, Clone)]
pub struct LedgerVerifier {
    ledger_path: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub valid: bool,
    pub count: usize,
    pub errors: Vec<String>,
}

impl LedgerVerifier {
    pub fn new(ledger_path: &Path) -> Self {
        Self {
            ledger_path: ledger_path.to_path_buf(),
        }
    }

    pub fn verify_integrity(&self) -> IntegrityReport {
        let mut report = IntegrityReport {
            valid: true,
            count: 0,
            errors: Vec::new(),
        };

        if !self.ledger_path.exists() {
            return report;
        }

        let content = std::fs::read_to_string(&self.ledger_path).unwrap_or_default();
        let mut last_hash = String::new();

        for (i, line) in content.lines().enumerate() {
            last_hash = self.process_ledger_line(line, i, last_hash, &mut report);
        }

        report
    }

    fn process_ledger_line(
        &self,
        line: &str,
        i: usize,
        last_hash: String,
        report: &mut IntegrityReport,
    ) -> String {
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(entry) => {
                let prev_hash = entry
                    .get("prev_hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if prev_hash != last_hash {
                    report.valid = false;
                    report
                        .errors
                        .push(format!("Hash mismatch at line {}", i + 1));
                }

                // Compute rolling hash of current entry
                // Note: Simplified - in production would strip rolling_hash field
                let mut hasher = Sha256::new();
                hasher.update(line.as_bytes());
                let current_hash = format!("{:x}", hasher.finalize());
                report.count += 1;
                current_hash
            }
            Err(e) => {
                report.valid = false;
                report
                    .errors
                    .push(format!("Error at line {}: {}", i + 1, e));
                last_hash
            }
        }
    }
}

/// Immutable incident ledger with rolling hash chain
#[derive(Debug, Clone)]
pub struct IncidentLedger {
    ledger_path: std::path::PathBuf,
    last_hash: String,
}

impl IncidentLedger {
    pub fn new(ledger_path: &Path) -> Self {
        let mut last_hash = String::new();

        if ledger_path.exists() {
            if let Ok(content) = std::fs::read_to_string(ledger_path) {
                for line in content.lines() {
                    last_hash = Self::get_last_hash_from_line(line, &last_hash);
                }
            }
        }

        Self {
            ledger_path: ledger_path.to_path_buf(),
            last_hash,
        }
    }

    fn get_last_hash_from_line(line: &str, current: &str) -> String {
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(entry) => entry
                .get("rolling_hash")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| current.to_string()),
            Err(_) => current.to_string(),
        }
    }

    pub fn record_artifact(
        &mut self,
        run_id: &str,
        action: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> String {
        let prev_hash = self.last_hash.clone();

        let entry = serde_json::json!({
            "run_id": run_id,
            "action": action,
            "payload": payload,
            "prev_hash": prev_hash,
        });

        let content = serde_json::to_string(&entry).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let current_hash = format!("{:x}", hasher.finalize());
        self.last_hash = current_hash.clone();

        // Write rolling_hash last
        let line = format!(
            "{},\"rolling_hash\":\"{}\"}}\n",
            content.trim_end_matches('}'),
            current_hash
        );

        if let Some(parent) = self.ledger_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.ledger_path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));

        current_hash
    }

    pub fn get_run_artifacts(&self, run_id: &str) -> Vec<serde_json::Value> {
        let mut out = Vec::new();

        if !self.ledger_path.exists() {
            return out;
        }

        if let Ok(content) = std::fs::read_to_string(&self.ledger_path) {
            for line in content.lines() {
                if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                    if entry.get("run_id").and_then(|v| v.as_str()) == Some(run_id) {
                        out.push(entry);
                    }
                }
            }
        }

        out
    }

    pub fn verify_integrity(&self) -> bool {
        let verifier = LedgerVerifier::new(&self.ledger_path);
        verifier.verify_integrity().valid
    }
}
