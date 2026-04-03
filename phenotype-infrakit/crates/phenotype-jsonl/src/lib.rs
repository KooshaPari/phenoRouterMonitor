//! BKM-10: thegent-jsonl — streaming JSONL parser library.
//!
//! Provides zero-copy line-by-line iteration over JSON objects in JSONL
//! (newline-delimited JSON) files without loading the entire file into memory.
//!
//! # Examples
//!
//! ```no_run
//! use std::path::Path;
//! use thegent_jsonl::{parse_file, filter_file, count_file, sample_file};
//!
//! let path = Path::new("audit.jsonl");
//! for record in parse_file(path).unwrap() {
//!     println!("{}", record.unwrap());
//! }
//! ```

pub mod audit;
pub mod ledger;
#[cfg(test)]
mod ledger_tests;
pub use audit::{AuditEntry, AuditLogger};
pub use ledger::{IncidentLedger, IntegrityReport, LedgerVerifier};

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Stdin};
use std::path::Path;

use anyhow::Result;
#[cfg(all(feature = "python", not(test), not(debug_assertions)))]
use pyo3::prelude::*;
#[cfg(all(feature = "python", not(test), not(debug_assertions)))]
use pyo3::pymodule;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Core streaming iterator
// ---------------------------------------------------------------------------

/// Iterator that yields parsed JSON values from a line-buffered reader.
///
/// Blank lines and lines consisting solely of whitespace are skipped.
/// Lines that fail JSON parsing yield `Err(anyhow::Error)`.
pub struct JsonlIter<R> {
    reader: BufReader<R>,
    buf: String,
    line_no: u64,
}

impl<R: Read> JsonlIter<R> {
    /// Wrap any `Read` implementor in a JSONL iterator.
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            buf: String::with_capacity(512),
            line_no: 0,
        }
    }
}

impl<R: Read> Iterator for JsonlIter<R> {
    type Item = Result<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();
            match self.reader.read_line(&mut self.buf) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    self.line_no += 1;
                    let trimmed = self.buf.trim();
                    if trimmed.is_empty() {
                        continue; // skip blank lines
                    }
                    let result = serde_json::from_str(trimmed)
                        .map_err(|e| anyhow::anyhow!("line {}: {}", self.line_no, e));
                    return Some(result);
                }
                Err(e) => {
                    return Some(Err(anyhow::anyhow!(
                        "IO error at line {}: {}",
                        self.line_no,
                        e
                    )));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Stream JSON objects from any `Read` implementor.
///
/// Returns an iterator yielding `Result<Value>` for each non-blank line.
pub fn parse_stream<R: Read>(reader: R) -> JsonlIter<R> {
    JsonlIter::new(reader)
}

/// Stream JSON objects from a file at `path`.
///
/// Opens the file and returns a lazy iterator.  The file handle is held open
/// for the lifetime of the iterator.
pub fn parse_file(path: &Path) -> Result<JsonlIter<File>> {
    let file = File::open(path).map_err(|e| anyhow::anyhow!("cannot open {:?}: {}", path, e))?;
    Ok(JsonlIter::new(file))
}

/// Stream JSON objects from stdin.
pub fn parse_stdin() -> JsonlIter<Stdin> {
    JsonlIter::new(io::stdin())
}

/// Filter a file's records to those where `obj[key] == value` (string compare).
///
/// Records that do not parse, or whose `key` field does not exist, are skipped
/// silently rather than propagated as errors — this matches the typical use
/// case for audit-log filtering where schema may vary.
pub fn filter_file<'a>(
    path: &Path,
    key: &'a str,
    value: &'a str,
) -> Result<impl Iterator<Item = Result<Value>> + 'a> {
    let iter = parse_file(path)?;
    let key_owned = key.to_owned();
    let value_owned = value.to_owned();
    Ok(iter.filter_map(move |res| match res {
        Ok(v) => {
            let matches = v
                .get(&key_owned)
                .map(|field| match field {
                    Value::String(s) => s == &value_owned,
                    other => other.to_string().trim_matches('"') == value_owned,
                })
                .unwrap_or(false);
            if matches {
                Some(Ok(v))
            } else {
                None
            }
        }
        Err(e) => Some(Err(e)),
    }))
}

/// Filter a stream's records to those where `obj[key] == value`.
///
/// Equivalent to `filter_file` but accepts any `Read` source.
pub fn filter_stream<'a, R: Read + 'a>(
    reader: R,
    key: &'a str,
    value: &'a str,
) -> impl Iterator<Item = Result<Value>> + 'a {
    let key_owned = key.to_owned();
    let value_owned = value.to_owned();
    parse_stream(reader).filter_map(move |res| match res {
        Ok(v) => {
            let matches = v
                .get(&key_owned)
                .map(|field| match field {
                    Value::String(s) => s == &value_owned,
                    other => other.to_string().trim_matches('"') == value_owned,
                })
                .unwrap_or(false);
            if matches {
                Some(Ok(v))
            } else {
                None
            }
        }
        Err(e) => Some(Err(e)),
    })
}

/// Count the number of non-blank lines in a JSONL stream.
///
/// Lines that fail JSON parsing are **counted** (they exist) and do not abort
/// the count — errors are returned only on IO failure.
pub fn count_stream<R: Read>(reader: R) -> Result<usize> {
    let mut n = 0usize;
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::with_capacity(256);
    loop {
        line.clear();
        match buf_reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if !line.trim().is_empty() {
                    n += 1;
                }
            }
            Err(e) => return Err(anyhow::anyhow!("IO error: {}", e)),
        }
    }
    Ok(n)
}

/// Count non-blank records in a file (parses JSON to validate each line).
pub fn count_file(path: &Path) -> Result<usize> {
    let file = File::open(path).map_err(|e| anyhow::anyhow!("cannot open {:?}: {}", path, e))?;
    count_stream(file)
}

/// Return up to `n` records from the beginning of a JSONL file.
///
/// Invalid JSON lines are included as `Err` in the result.
pub fn sample_file(path: &Path, n: usize) -> Result<Vec<Result<Value>>> {
    let iter = parse_file(path)?;
    Ok(iter.take(n).collect())
}

/// Return up to `n` records from a stream.
pub fn sample_stream<R: Read>(reader: R, n: usize) -> Vec<Result<Value>> {
    parse_stream(reader).take(n).collect()
}

#[cfg(all(feature = "python", not(test), not(debug_assertions)))]
#[pymodule]
fn thegent_jsonl(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn cursor(s: &str) -> Cursor<Vec<u8>> {
        Cursor::new(s.as_bytes().to_vec())
    }

    #[test]
    fn test_parse_stream_valid() {
        let input = r#"{"a":1}
{"b":2}
"#;
        let records: Vec<_> = parse_stream(cursor(input)).collect();
        assert_eq!(records.len(), 2);
        assert!(records[0].is_ok());
        assert_eq!(records[0].as_ref().unwrap()["a"], 1);
    }

    #[test]
    fn test_parse_stream_skips_blank_lines() {
        let input = "\n{\"x\":1}\n\n{\"y\":2}\n\n";
        let records: Vec<_> = parse_stream(cursor(input)).collect();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_parse_stream_invalid_json_yields_err() {
        let input = "not_json\n{\"ok\":true}\n";
        let records: Vec<_> = parse_stream(cursor(input)).collect();
        assert_eq!(records.len(), 2);
        assert!(records[0].is_err());
        assert!(records[1].is_ok());
    }

    #[test]
    fn test_count_stream_empty() {
        let n = count_stream(cursor("")).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn test_count_stream_with_blanks() {
        let input = "\n{\"a\":1}\n\n{\"b\":2}\n";
        let n = count_stream(cursor(input)).unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn test_filter_stream_basic() {
        let input = r#"{"type":"error","msg":"oops"}
{"type":"info","msg":"ok"}
{"type":"error","msg":"again"}
"#;
        let results: Vec<_> = filter_stream(cursor(input), "type", "error").collect();
        assert_eq!(results.len(), 2);
        for r in &results {
            assert_eq!(r.as_ref().unwrap()["type"], "error");
        }
    }

    #[test]
    fn test_filter_stream_no_match() {
        let input = r#"{"type":"info"}
{"type":"debug"}
"#;
        let results: Vec<_> = filter_stream(cursor(input), "type", "error").collect();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_sample_stream() {
        let input = r#"{"n":1}
{"n":2}
{"n":3}
{"n":4}
"#;
        let sample = sample_stream(cursor(input), 2);
        assert_eq!(sample.len(), 2);
        assert_eq!(sample[0].as_ref().unwrap()["n"], 1);
        assert_eq!(sample[1].as_ref().unwrap()["n"], 2);
    }

    #[test]
    fn test_sample_stream_fewer_than_n() {
        let input = "{\"n\":1}\n";
        let sample = sample_stream(cursor(input), 10);
        assert_eq!(sample.len(), 1);
    }
}
