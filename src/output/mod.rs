pub mod color;
pub mod csv;
pub mod json;
pub mod table;

use crate::context::OutputFormat;

/// The universal return type for commands.
/// Commands produce data; the output layer formats it.
pub enum CommandOutput {
    /// Tabular data: headers + rows.
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    /// Single-resource detail view: key-value pairs.
    Detail { fields: Vec<(String, String)> },
    /// Raw message (e.g. "Monitor deleted.").
    Message(String),
    /// Pre-formatted output (passed through all renderers as-is).
    Raw(String),
    /// No output (e.g. help was already printed).
    Empty,
}

pub fn render(output: &CommandOutput, format: OutputFormat, no_color: bool) -> String {
    match format {
        OutputFormat::Json => json::render(output),
        OutputFormat::Csv => csv::render(output),
        OutputFormat::Table => table::render(output, no_color),
    }
}
