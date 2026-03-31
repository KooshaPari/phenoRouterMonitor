//! Command context abstraction for reducing boilerplate across CLI handlers.
//!
//! Encapsulates common command patterns:
//! - Logger setup and timing
//! - Storage/VCS port access
//! - Output formatting (table/JSON)
//! - Error context wrapping
//! - Telemetry collection
//!
//! Traceability: WP11-T060, T065 / Cross-command consolidation

use std::fmt::Debug;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::Serialize;

use agileplus_domain::ports::{StoragePort, VcsPort};

/// Output format preference for command results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable table format.
    Table,
    /// Machine-readable JSON format.
    Json,
}

impl OutputFormat {
    /// Parse from string (case-insensitive).
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "table" => Ok(OutputFormat::Table),
            other => anyhow::bail!(
                "unsupported output format: '{}' (use 'table' or 'json')",
                other
            ),
        }
    }
}

/// Execution timing and telemetry for a command.
#[derive(Debug, Clone)]
pub struct CommandTelemetry {
    /// Wall-clock duration of command execution.
    pub duration: Duration,
}

impl CommandTelemetry {
    /// Get duration in milliseconds.
    pub fn duration_ms(&self) -> u128 {
        self.duration.as_millis()
    }
}

/// Command execution context combining I/O, configuration, and telemetry.
///
/// Generic over storage and VCS adapters to maintain type safety and
/// testability while reducing boilerplate across all command handlers.
///
/// # Example
///
/// ```ignore
/// let ctx = CommandContext::new("my-command", storage, vcs);
/// ctx.log_start();
/// // ... perform command logic ...
/// ctx.log_complete();
/// ```
pub struct CommandContext<'a, S: StoragePort, V: VcsPort> {
    /// Descriptive command name for logging.
    command_name: String,

    /// Storage port (database access).
    storage: &'a S,

    /// VCS port (git access).
    vcs: &'a V,

    /// Output format preference.
    output_format: OutputFormat,

    /// Command start time for telemetry.
    start_time: Instant,

    /// Collected telemetry data.
    telemetry: CommandTelemetry,
}

impl<'a, S: StoragePort, V: VcsPort> CommandContext<'a, S, V> {
    /// Create a new command context.
    ///
    /// Starts timing immediately.
    ///
    /// # Arguments
    /// * `command_name` - Descriptive name for logging (e.g., "specify", "ship")
    /// * `storage` - Storage port reference
    /// * `vcs` - VCS port reference
    pub fn new(command_name: impl Into<String>, storage: &'a S, vcs: &'a V) -> Self {
        Self {
            command_name: command_name.into(),
            storage,
            vcs,
            output_format: OutputFormat::Table,
            start_time: Instant::now(),
            telemetry: CommandTelemetry {
                duration: Duration::ZERO,
            },
        }
    }

    /// Set the preferred output format.
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Parse and set output format from a string argument.
    pub fn with_format_str(mut self, format_str: &str) -> Result<Self> {
        self.output_format = OutputFormat::from_str(format_str)?;
        Ok(self)
    }

    /// Get the storage port.
    pub fn storage(&self) -> &'a S {
        self.storage
    }

    /// Get the VCS port.
    pub fn vcs(&self) -> &'a V {
        self.vcs
    }

    /// Get the output format.
    pub fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    /// Check if output should be JSON.
    pub fn is_json(&self) -> bool {
        self.output_format == OutputFormat::Json
    }

    /// Check if output should be table format.
    pub fn is_table(&self) -> bool {
        self.output_format == OutputFormat::Table
    }

    /// Log command start to tracing system.
    pub fn log_start(&self) {
        tracing::debug!(command = %self.command_name, "starting command");
    }

    /// Update telemetry and log command completion.
    pub fn log_complete(&mut self) {
        self.telemetry.duration = self.start_time.elapsed();
        tracing::info!(
            command = %self.command_name,
            elapsed_ms = self.telemetry.duration_ms(),
            "command completed successfully"
        );
    }

    /// Log command completion with additional context fields.
    ///
    /// # Arguments
    /// * `extra_fields` - Closure that receives a span builder to add extra fields
    pub fn log_complete_with<F>(&mut self, extra_fields: F)
    where
        F: FnOnce(&mut tracing::Span),
    {
        self.telemetry.duration = self.start_time.elapsed();
        let span = tracing::info_span!(
            "command_complete",
            command = %self.command_name,
            elapsed_ms = self.telemetry.duration_ms()
        );
        let mut guard = span.enter();
        let _ = guard;
        tracing::info!("command completed successfully");
    }

    /// Log an error with command context.
    pub fn log_error(&self, error: &dyn std::error::Error) {
        tracing::error!(
            command = %self.command_name,
            error = %error,
            "command failed"
        );
    }

    /// Format and print a result as JSON or table, depending on output format.
    ///
    /// For JSON: serializes the value and prints it pretty-printed.
    /// For table: calls the provided formatter.
    ///
    /// # Arguments
    /// * `value` - The serializable value to output
    /// * `table_formatter` - Closure that formats value for table output
    pub fn output<T: Serialize>(
        &self,
        value: &T,
        table_formatter: impl FnOnce(&T) -> Result<()>,
    ) -> Result<()> {
        if self.is_json() {
            let json = serde_json::to_string_pretty(value)?;
            println!("{json}");
        } else {
            table_formatter(value)?;
        }
        Ok(())
    }

    /// Print JSON output directly from a value.
    pub fn output_json<T: Serialize>(&self, value: &T) -> Result<()> {
        let json = serde_json::to_string_pretty(value)?;
        println!("{json}");
        Ok(())
    }

    /// Print table output using a custom formatter.
    pub fn output_table<T>(
        &self,
        value: &T,
        formatter: impl FnOnce(&T) -> Result<()>,
    ) -> Result<()> {
        formatter(value)
    }

    /// Get reference to telemetry data.
    pub fn telemetry(&self) -> &CommandTelemetry {
        &self.telemetry
    }

    /// Consume context and return telemetry for external tracking.
    pub fn into_telemetry(mut self) -> CommandTelemetry {
        self.telemetry.duration = self.start_time.elapsed();
        self.telemetry
    }
}

/// Simplified context for commands that don't need both storage and VCS.
///
/// Use this when you only need storage (e.g., list, query commands).
pub struct StorageOnlyContext<'a, S: StoragePort> {
    command_name: String,
    storage: &'a S,
    output_format: OutputFormat,
    start_time: Instant,
    telemetry: CommandTelemetry,
}

impl<'a, S: StoragePort> StorageOnlyContext<'a, S> {
    /// Create a new storage-only context.
    pub fn new(command_name: impl Into<String>, storage: &'a S) -> Self {
        Self {
            command_name: command_name.into(),
            storage,
            output_format: OutputFormat::Table,
            start_time: Instant::now(),
            telemetry: CommandTelemetry {
                duration: Duration::ZERO,
            },
        }
    }

    /// Set the preferred output format.
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Parse and set output format from a string argument.
    pub fn with_format_str(mut self, format_str: &str) -> Result<Self> {
        self.output_format = OutputFormat::from_str(format_str)?;
        Ok(self)
    }

    /// Get the storage port.
    pub fn storage(&self) -> &'a S {
        self.storage
    }

    /// Get the output format.
    pub fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    /// Check if output should be JSON.
    pub fn is_json(&self) -> bool {
        self.output_format == OutputFormat::Json
    }

    /// Check if output should be table format.
    pub fn is_table(&self) -> bool {
        self.output_format == OutputFormat::Table
    }

    /// Log command start.
    pub fn log_start(&self) {
        tracing::debug!(command = %self.command_name, "starting command");
    }

    /// Update telemetry and log completion.
    pub fn log_complete(&mut self) {
        self.telemetry.duration = self.start_time.elapsed();
        tracing::info!(
            command = %self.command_name,
            elapsed_ms = self.telemetry.duration_ms(),
            "command completed successfully"
        );
    }

    /// Log an error.
    pub fn log_error(&self, error: &dyn std::error::Error) {
        tracing::error!(
            command = %self.command_name,
            error = %error,
            "command failed"
        );
    }

    /// Format and print output.
    pub fn output<T: Serialize>(
        &self,
        value: &T,
        table_formatter: impl FnOnce(&T) -> Result<()>,
    ) -> Result<()> {
        if self.is_json() {
            let json = serde_json::to_string_pretty(value)?;
            println!("{json}");
        } else {
            table_formatter(value)?;
        }
        Ok(())
    }

    /// Get telemetry reference.
    pub fn telemetry(&self) -> &CommandTelemetry {
        &self.telemetry
    }

    /// Consume and return telemetry.
    pub fn into_telemetry(mut self) -> CommandTelemetry {
        self.telemetry.duration = self.start_time.elapsed();
        self.telemetry
    }
}

/// Helper trait for types that can be formatted as table output.
///
/// Implement this on your result types to integrate with CommandContext.
pub trait TableFormattable {
    /// Format self as table output, writing to stdout.
    fn format_table(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_format_from_str_json() {
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("JSON").unwrap(), OutputFormat::Json);
    }

    #[test]
    fn output_format_from_str_table() {
        assert_eq!(
            OutputFormat::from_str("table").unwrap(),
            OutputFormat::Table
        );
        assert_eq!(
            OutputFormat::from_str("TABLE").unwrap(),
            OutputFormat::Table
        );
    }

    #[test]
    fn output_format_from_str_invalid() {
        assert!(OutputFormat::from_str("xml").is_err());
    }

    #[test]
    fn telemetry_duration_ms() {
        let telemetry = CommandTelemetry {
            duration: Duration::from_millis(123),
        };
        assert_eq!(telemetry.duration_ms(), 123);
    }
}
