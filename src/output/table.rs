use super::CommandOutput;
use super::color;

pub fn render(output: &CommandOutput, no_color: bool) -> String {
    match output {
        CommandOutput::Table { headers, rows } => {
            if rows.is_empty() {
                return if no_color {
                    "No results found.".to_string()
                } else {
                    color::dim("No results found.")
                };
            }
            render_table(headers, rows, no_color)
        }
        CommandOutput::Detail { fields } => {
            if fields.is_empty() {
                return if no_color {
                    "No details available.".to_string()
                } else {
                    color::dim("No details available.")
                };
            }
            render_detail(fields, no_color)
        }
        CommandOutput::Message(msg) => msg.clone(),
        CommandOutput::Empty => String::new(),
    }
}

fn render_detail(fields: &[(String, String)], no_color: bool) -> String {
    let max_key = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    fields
        .iter()
        .map(|(k, v)| {
            let key = if no_color {
                format!("{:<width$}", k, width = max_key)
            } else {
                color::bold(&format!("{:<width$}", k, width = max_key))
            };
            let val = colorize_value(k, v, no_color);
            format!("{key}  {val}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_table(headers: &[String], rows: &[Vec<String>], no_color: bool) -> String {
    let col_count = headers.len();

    // Calculate max width per column (based on raw text, not ANSI codes)
    let mut widths = vec![0usize; col_count];
    for (i, h) in headers.iter().enumerate() {
        widths[i] = h.len();
    }
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let mut out = String::new();

    // Header
    let header_line: String = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let padded = format!("{:<width$}", h, width = widths[i]);
            if no_color {
                padded
            } else {
                color::bold(&padded)
            }
        })
        .collect::<Vec<_>>()
        .join("  ");
    out.push_str(&header_line);
    out.push('\n');

    // Separator
    let sep_raw: String = widths
        .iter()
        .map(|w| "─".repeat(*w))
        .collect::<Vec<_>>()
        .join("──");
    if no_color {
        out.push_str(&sep_raw);
    } else {
        out.push_str(&color::dim(&sep_raw));
    }
    out.push('\n');

    // Rows
    for row in rows {
        let line: String = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let w = widths.get(i).copied().unwrap_or(0);
                let header = headers.get(i).map(|s| s.as_str()).unwrap_or("");
                let colored = colorize_value(header, cell, no_color);
                // Pad after colorizing: we need to account for ANSI codes in length
                let visible_len = cell.len();
                let padding = w.saturating_sub(visible_len);
                format!("{colored}{}", " ".repeat(padding))
            })
            .collect::<Vec<_>>()
            .join("  ");
        out.push_str(&line);
        out.push('\n');
    }

    // Trim trailing newline
    out.truncate(out.trim_end().len());
    out
}

/// Apply color to cell values based on the column header.
fn colorize_value(header: &str, value: &str, no_color: bool) -> String {
    if no_color || value == "-" {
        return value.to_string();
    }
    match header {
        "Status" => color::status(value),
        "Uptime token" | "Telemetry token" => {
            if value == "not set" {
                color::red(value)
            } else {
                color::green(value)
            }
        }
        _ => value.to_string(),
    }
}
