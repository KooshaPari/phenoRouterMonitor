//! Audit logging for routing decisions.
//!
//! Appends to `routing_audit.jsonl` with SHA-256 hash chaining
//! per ADR-015 pattern (EvidenceLedger canonical approach):
//!
//!   canonical_json = sort_keys(record - "hash" field)
//!   record["hash"] = sha256(canonical_json)
//!   record["prev_hash"] = hash_of_previous_record

// @trace WL-074
use sha2::{Digest, Sha256};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read as _, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// A single routing decision audit record.
///
/// Fields follow ADR-015 naming conventions with routing-specific additions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    /// ISO-8601 UTC timestamp.
    pub timestamp: String,
    /// Unique identifier for this routing decision.
    pub decision_id: String,
    /// Provider dispatched to (e.g. "lifecycle", "thegent").
    pub provider: String,
    /// Model used (e.g. "gemini-3-flash", "claude-sonnet-4.6").
    pub model: String,
    /// Execution latency in milliseconds.
    pub latency_ms: u64,
    /// Estimated cost in USD.
    pub cost: f64,
    /// SHA-256 hash of the previous record (empty string for genesis).
    pub prev_hash: String,
    /// SHA-256 hash of this record (all fields except `hash`, sorted keys).
    pub hash: String,
}

impl AuditRecord {
    /// Create a new audit record (hash fields are computed internally).
    pub fn new(provider: String, model: String, latency_ms: u64, cost: f64) -> Self {
        let timestamp = iso8601_now();
        let decision_id = Uuid::new_v4().to_string();

        let mut record = Self {
            timestamp,
            decision_id,
            provider,
            model,
            latency_ms,
            cost,
            prev_hash: String::new(),
            hash: String::new(),
        };

        // Compute hash over all fields except `hash` itself.
        record.hash = compute_hash(&record);
        record
    }

    /// Rebuild with a prev_hash link and recompute the hash.
    pub fn with_prev_hash(mut self, prev_hash: String) -> Self {
        self.prev_hash = prev_hash;
        self.hash = compute_hash(&self);
        self
    }
}

/// Compute the canonical SHA-256 hash for an AuditRecord following ADR-015.
///
/// Uses BTreeMap for deterministic key ordering (sort_keys equivalent).
/// Excludes the `hash` field itself from the computation.
fn compute_hash(record: &AuditRecord) -> String {
    let mut map = BTreeMap::new();
    map.insert(
        "timestamp",
        serde_json::Value::String(record.timestamp.clone()),
    );
    map.insert(
        "decision_id",
        serde_json::Value::String(record.decision_id.clone()),
    );
    map.insert(
        "provider",
        serde_json::Value::String(record.provider.clone()),
    );
    map.insert("model", serde_json::Value::String(record.model.clone()));
    map.insert(
        "latency_ms",
        serde_json::Value::Number(record.latency_ms.into()),
    );
    map.insert("cost", serde_json::json!(record.cost));
    map.insert(
        "prev_hash",
        serde_json::Value::String(record.prev_hash.clone()),
    );

    let canonical = serde_json::to_string(&map).expect("BTreeMap serialization never fails");
    sha256_hex(canonical.as_bytes())
}

/// Compute SHA-256 of `input` and return as a lowercase hex string.
///
/// Uses the `sha2` crate (SIMD-accelerated, 10-20x faster than hand-rolled).
// @trace WL-074
fn sha256_hex(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    hex::encode(result)
}

fn iso8601_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Format as ISO-8601 UTC (YYYY-MM-DDTHH:MM:SSZ).
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // Simplified date math: days since epoch → date.
    let year = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year.0, year.1, year.2, h, m, s
    )
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    let mut remaining = days;
    let mut year = 1970u32;
    loop {
        let leap = is_leap(year);
        let days_in_year = if leap { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [u32; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u32;
    for &md in &month_days {
        if remaining < md as u64 {
            break;
        }
        remaining -= md as u64;
        month += 1;
    }
    (year, month, remaining as u32 + 1)
}

fn is_leap(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// Append-only audit logger for routing decisions.
///
/// Thread-safe via internal Mutex. Maintains ADR-015 hash chain:
/// each record contains `prev_hash` pointing to the previous record's `hash`.
///
/// The file is opened once at construction and held open as a `BufWriter<File>`
/// for the lifetime of the logger. `flush()` is called after every record to
/// ensure durability; the buffer is also flushed on `Drop`.
// @trace WL-075
pub struct AuditLogger {
    /// Mutex protects the last-hash state and the open file writer together.
    state: Mutex<AuditState>,
}

// @trace WL-075
struct AuditState {
    last_hash: String,
    writer: BufWriter<File>,
    path: PathBuf,
}

impl AuditLogger {
    /// Create a new logger writing to `path`.
    ///
    /// Creates parent directories if they do not exist.
    /// If the file already exists, reads the last line to restore the chain head.
    /// Panics if the file cannot be created/opened (loud failure per governance).
    // @trace WL-075
    pub fn new(path: PathBuf) -> Self {
        // Create parent directories before opening the file.
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .expect("AuditLogger: failed to create parent directories");
        }

        let last_hash = Self::read_last_hash(&path);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .expect("AuditLogger: failed to open audit log file");

        Self {
            state: Mutex::new(AuditState {
                last_hash,
                writer: BufWriter::new(file),
                path,
            }),
        }
    }

    /// Append an audit record, linking it into the hash chain.
    ///
    /// Returns the `decision_id` of the appended record.
    /// Flushes the `BufWriter` after each record for durability.
    // @trace WL-075
    pub fn append(&self, record: &AuditRecord) -> Result<String, String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;

        // Link record into chain.
        let linked = record.clone().with_prev_hash(state.last_hash.clone());

        // Serialize to JSON line.
        let line = serde_json::to_string(&linked).map_err(|e| e.to_string())?;

        // Write to the held-open BufWriter.
        writeln!(state.writer, "{}", line).map_err(|e| e.to_string())?;

        // Flush after each record to ensure durability.
        state.writer.flush().map_err(|e| e.to_string())?;

        // Update chain head.
        state.last_hash = linked.hash.clone();
        Ok(linked.decision_id)
    }

    /// Read all records from the log file.
    ///
    /// Returns records in append order. Malformed lines are skipped.
    pub fn read_all(&self) -> Vec<AuditRecord> {
        let state = self.state.lock().expect("AuditLogger mutex poisoned");
        Self::read_records(&state.path)
    }

    /// Verify the hash chain integrity.
    ///
    /// Returns Ok(count) if all N records pass, Err(message) on first violation.
    pub fn verify_chain(&self) -> Result<usize, String> {
        let records = self.read_all();
        let mut prev_hash = String::new();
        for (i, record) in records.iter().enumerate() {
            // Verify prev_hash link.
            if record.prev_hash != prev_hash {
                return Err(format!(
                    "Chain broken at record {}: expected prev_hash={} got {}",
                    i, prev_hash, record.prev_hash
                ));
            }
            // Recompute hash.
            let expected = compute_hash(record);
            if record.hash != expected {
                return Err(format!(
                    "Hash mismatch at record {} (decision_id={}): expected {} got {}",
                    i, record.decision_id, expected, record.hash
                ));
            }
            prev_hash = record.hash.clone();
        }
        Ok(records.len())
    }

    /// Read only the last line of the audit log to recover the chain head.
    ///
    /// O(1) in file size: seeks to end, scans backwards for the last newline,
    /// then reads just that final JSON line.
    // @trace WL-075
    fn read_last_hash(path: &PathBuf) -> String {
        if !path.exists() {
            return String::new();
        }
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return String::new(),
        };
        let file_len = match file.seek(SeekFrom::End(0)) {
            Ok(len) => len,
            Err(_) => return String::new(),
        };
        if file_len == 0 {
            return String::new();
        }

        // Read a tail chunk (up to 8 KiB) which is more than enough for one JSONL record.
        let chunk_size = 8192u64.min(file_len);
        let start = file_len - chunk_size;
        file.seek(SeekFrom::Start(start))
            .expect("AuditLogger: seek failed in read_last_hash");

        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .expect("AuditLogger: read failed in read_last_hash");

        // Find the last non-empty line.
        buf.lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .and_then(|l| serde_json::from_str::<AuditRecord>(l).ok())
            .map(|r| r.hash)
            .unwrap_or_default()
    }

    fn read_records(path: &PathBuf) -> Vec<AuditRecord> {
        if !path.exists() {
            return vec![];
        }
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return vec![],
        };
        BufReader::new(file)
            .lines()
            .map_while(Result::ok)
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str::<AuditRecord>(&l).ok())
            .collect()
    }
}

/// Flush the `BufWriter` on drop so buffered audit records are not lost.
// @trace WL-075
impl Drop for AuditLogger {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            let _ = state.writer.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_audit_record_hash_stable() {
        let r1 = AuditRecord::new(
            "lifecycle".to_string(),
            "gemini-3-flash".to_string(),
            10,
            0.001,
        );
        let r2 = AuditRecord::new(
            "lifecycle".to_string(),
            "gemini-3-flash".to_string(),
            10,
            0.001,
        );
        // Different timestamps/decision_ids → different hashes.
        // But same structure is verifiable.
        assert!(!r1.hash.is_empty());
        assert!(!r2.hash.is_empty());
        assert_eq!(r1.hash.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_audit_record_hash_chain() {
        let r = AuditRecord::new(
            "thegent".to_string(),
            "claude-sonnet-4.6".to_string(),
            50,
            0.01,
        );
        let r2 = r.clone().with_prev_hash(r.hash.clone());
        assert_eq!(r2.prev_hash, r.hash);
        assert_ne!(r2.hash, r.hash); // Different because prev_hash changed.
    }

    #[test]
    fn test_audit_logger_append_and_read() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("routing_audit.jsonl");
        let logger = AuditLogger::new(path.clone());

        let r = AuditRecord::new(
            "lifecycle".to_string(),
            "gemini-3-flash".to_string(),
            5,
            0.0001,
        );
        logger.append(&r).unwrap();

        let records = logger.read_all();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].provider, "lifecycle");
    }

    #[test]
    fn test_audit_logger_chain_verification() {
        let dir = tempdir().unwrap();
        let logger = AuditLogger::new(dir.path().join("routing_audit.jsonl"));

        for i in 0..5 {
            let r = AuditRecord::new(
                "lifecycle".to_string(),
                "gemini-3-flash".to_string(),
                i * 10,
                0.0001 * i as f64,
            );
            logger.append(&r).unwrap();
        }

        let result = logger.verify_chain();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
    }

    #[test]
    fn test_audit_logger_first_record_empty_prev_hash() {
        let dir = tempdir().unwrap();
        let logger = AuditLogger::new(dir.path().join("routing_audit.jsonl"));

        let r = AuditRecord::new(
            "thegent".to_string(),
            "claude-sonnet-4.6".to_string(),
            100,
            0.01,
        );
        logger.append(&r).unwrap();

        let records = logger.read_all();
        assert_eq!(records[0].prev_hash, "");
    }

    #[test]
    fn test_audit_logger_chain_links() {
        let dir = tempdir().unwrap();
        let logger = AuditLogger::new(dir.path().join("routing_audit.jsonl"));

        let r1 = AuditRecord::new(
            "lifecycle".to_string(),
            "gemini-3-flash".to_string(),
            10,
            0.001,
        );
        let r2 = AuditRecord::new(
            "thegent".to_string(),
            "claude-sonnet-4.6".to_string(),
            50,
            0.005,
        );
        logger.append(&r1).unwrap();
        logger.append(&r2).unwrap();

        let records = logger.read_all();
        assert_eq!(records.len(), 2);
        // Second record's prev_hash = first record's hash.
        assert_eq!(records[1].prev_hash, records[0].hash);
    }

    #[test]
    fn test_audit_logger_resumes_chain_on_reopen() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("routing_audit.jsonl");

        // First session.
        {
            let logger = AuditLogger::new(path.clone());
            let r = AuditRecord::new(
                "lifecycle".to_string(),
                "gemini-3-flash".to_string(),
                5,
                0.001,
            );
            logger.append(&r).unwrap();
        }

        // Second session: reads prior chain head.
        {
            let logger = AuditLogger::new(path.clone());
            let r = AuditRecord::new(
                "thegent".to_string(),
                "claude-sonnet-4.6".to_string(),
                20,
                0.002,
            );
            logger.append(&r).unwrap();
        }

        let logger = AuditLogger::new(path);
        let result = logger.verify_chain();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    // @trace FR-OPT-007
    #[test]
    fn test_sha256_known_value() {
        let hash = sha256_hex(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    // @trace FR-OPT-007
    #[test]
    fn test_sha256_hello_world() {
        let hash = sha256_hex(b"hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_audit_logger_creates_parent_dir() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("subdir").join("routing_audit.jsonl");
        let logger = AuditLogger::new(path.clone());

        let r = AuditRecord::new(
            "lifecycle".to_string(),
            "gemini-3-flash".to_string(),
            5,
            0.001,
        );
        logger.append(&r).unwrap();

        assert!(path.exists());
    }

    // @trace FR-OPT-008
    #[test]
    fn test_audit_logger_reuses_file_handle() {
        // Write 100 records through a single AuditLogger instance.
        // The file should exist and contain exactly 100 lines.
        // The BufWriter is held open for the lifetime of the logger,
        // so there is no per-record open() overhead.
        let dir = tempdir().unwrap();
        let path = dir.path().join("routing_audit.jsonl");
        let logger = AuditLogger::new(path.clone());

        for i in 0..100 {
            let r = AuditRecord::new(
                "lifecycle".to_string(),
                "gemini-3-flash".to_string(),
                i,
                i as f64,
            );
            logger.append(&r).unwrap();
        }

        // Verify all 100 records are present and chain is intact.
        let records = logger.read_all();
        assert_eq!(records.len(), 100);
        let chain_result = logger.verify_chain();
        assert!(
            chain_result.is_ok(),
            "chain verification failed: {:?}",
            chain_result.err()
        );
        assert_eq!(chain_result.unwrap(), 100);

        // Verify the raw file has exactly 100 lines.
        let content = std::fs::read_to_string(&path).unwrap();
        let line_count = content.lines().filter(|l| !l.trim().is_empty()).count();
        assert_eq!(line_count, 100);
    }

    // @trace FR-OPT-008
    #[test]
    fn test_read_last_hash_tail_read() {
        // Verify that reopening a logger with an existing file correctly
        // reads the last record's hash without parsing all records.
        let dir = tempdir().unwrap();
        let path = dir.path().join("routing_audit.jsonl");

        // Write 10 records in first session.
        let last_hash;
        {
            let logger = AuditLogger::new(path.clone());
            for i in 0..10 {
                let r = AuditRecord::new(
                    "lifecycle".to_string(),
                    "gemini-3-flash".to_string(),
                    i * 5,
                    0.001,
                );
                logger.append(&r).unwrap();
            }
            let records = logger.read_all();
            last_hash = records.last().unwrap().hash.clone();
        }

        // Reopen: the logger should pick up the last hash.
        let logger = AuditLogger::new(path);
        let r = AuditRecord::new(
            "thegent".to_string(),
            "claude-sonnet-4.6".to_string(),
            42,
            0.01,
        );
        logger.append(&r).unwrap();

        // The 11th record should chain from the 10th record's hash.
        let records = logger.read_all();
        assert_eq!(records.len(), 11);
        assert_eq!(records[10].prev_hash, last_hash);

        // Full chain should verify.
        let chain_result = logger.verify_chain();
        assert!(chain_result.is_ok());
        assert_eq!(chain_result.unwrap(), 11);
    }
}
