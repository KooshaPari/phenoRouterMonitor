//! JSON Lines parsing utilities

use std::io::{BufRead, Write};

/// Write a JSON Lines file
pub fn write_jsonl<W: Write, T: serde::Serialize>(
    writer: &mut W,
    items: &[T],
) -> anyhow::Result<()> {
    for item in items {
        let line = serde_json::to_string(item)?;
        writeln!(writer, "{}", line)?;
    }
    Ok(())
}

/// Read a JSON Lines file
pub fn read_jsonl<R: BufRead, T: serde::de::DeserializeOwned>(
    reader: R,
) -> anyhow::Result<Vec<T>> {
    let mut items = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let item: T = serde_json::from_str(&line)?;
        items.push(item);
    }
    Ok(items)
}
