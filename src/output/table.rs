use unicode_width::UnicodeWidthStr;

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
        CommandOutput::Raw(s) => s.clone(),
        CommandOutput::Empty => String::new(),
    }
}

fn render_detail(fields: &[(String, String)], no_color: bool) -> String {
    let max_key = fields
        .iter()
        .map(|(k, _)| display_width(k))
        .max()
        .unwrap_or(0);
    fields
        .iter()
        .map(|(k, v)| {
            let padded = pad_right(k, max_key);
            let key = if no_color {
                padded
            } else {
                color::bold(&padded)
            };
            let val = colorize_value(k, v, no_color);
            format!("{key}  {val}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

fn pad_right(s: &str, target_display_width: usize) -> String {
    let w = display_width(s);
    if w >= target_display_width {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(target_display_width - w))
    }
}

fn truncate(s: &str, max: usize) -> String {
    if display_width(s) <= max {
        return s.to_string();
    }
    if max <= 3 {
        return ".".repeat(max);
    }
    let target = max - 3;
    let mut end = 0;
    let mut w = 0;
    for (i, ch) in s.char_indices() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > target {
            break;
        }
        w += cw;
        end = i + ch.len_utf8();
    }
    format!("{}...", &s[..end])
}

fn term_width() -> usize {
    // Try to get terminal width, fall back to 120
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(1, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            return ws.ws_col as usize;
        }
    }
    120
}

fn compute_widths(headers: &[String], rows: &[Vec<String>]) -> Vec<usize> {
    let col_count = headers.len();
    let gap = 2;

    // Natural width = max of header and all cell display widths
    let mut natural = vec![0usize; col_count];
    for (i, h) in headers.iter().enumerate() {
        natural[i] = display_width(h);
    }
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                natural[i] = natural[i].max(display_width(cell));
            }
        }
    }

    let total_gaps = gap * col_count.saturating_sub(1);
    let budget = term_width().saturating_sub(total_gaps);
    let total_natural: usize = natural.iter().sum();

    if total_natural <= budget {
        return natural;
    }

    // Iteratively shrink: lock columns that fit, shrink the rest
    let mut widths = natural.clone();
    let mut locked = vec![false; col_count];
    let mut excess = total_natural.saturating_sub(budget);

    for _ in 0..col_count {
        if excess == 0 {
            break;
        }

        // Find shrinkable columns (unlocked and wider than their header)
        let shrinkable: Vec<usize> = (0..col_count)
            .filter(|&i| !locked[i] && widths[i] > display_width(&headers[i]))
            .collect();

        if shrinkable.is_empty() {
            break;
        }

        // Sort by width descending - shrink the widest first
        let max_w = shrinkable.iter().map(|&i| widths[i]).max().unwrap_or(0);
        let second_max = shrinkable
            .iter()
            .map(|&i| widths[i])
            .filter(|&w| w < max_w)
            .max()
            .unwrap_or_else(|| {
                shrinkable
                    .iter()
                    .map(|&i| display_width(&headers[i]))
                    .max()
                    .unwrap_or(0)
            });

        // Shrink all widest columns down toward second_max
        let widest: Vec<usize> = shrinkable
            .iter()
            .copied()
            .filter(|&i| widths[i] == max_w)
            .collect();
        let can_trim_each = max_w - second_max;
        let total_can_trim = can_trim_each * widest.len();

        if total_can_trim <= excess {
            for &i in &widest {
                widths[i] = second_max;
            }
            excess -= total_can_trim;
        } else {
            // Distribute the remaining excess evenly
            let per_col = excess / widest.len();
            let leftover = excess % widest.len();
            for (j, &i) in widest.iter().enumerate() {
                widths[i] -= per_col + if j < leftover { 1 } else { 0 };
            }
            excess = 0;
        }

        // Lock columns that hit their minimum
        for &i in &shrinkable {
            if widths[i] <= display_width(&headers[i]) {
                locked[i] = true;
            }
        }
    }

    widths
}

fn render_table(headers: &[String], rows: &[Vec<String>], no_color: bool) -> String {
    let col_count = headers.len();
    let widths = compute_widths(headers, rows);

    let mut out = String::new();

    // Header
    let header_line: String = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let padded = pad_right(h, widths[i]);
            if no_color {
                padded
            } else {
                color::bold(&padded)
            }
        })
        .collect::<Vec<_>>()
        .join("  ");
    out.push_str(header_line.trim_end());
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
                let display = truncate(cell, w);
                let colored = colorize_value(header, &display, no_color);
                if i == col_count - 1 {
                    colored
                } else {
                    let visible_len = display_width(&display);
                    let padding = w.saturating_sub(visible_len);
                    format!("{colored}{}", " ".repeat(padding))
                }
            })
            .collect::<Vec<_>>()
            .join("  ");
        out.push_str(line.trim_end());
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
        "Uptime token" | "Telemetry token" | "SQL connection" => {
            if value == "not set" {
                color::red(value)
            } else {
                color::green(value)
            }
        }
        "level" => color::level_badge(value),
        "dt" => color::dim(value),
        _ => value.to_string(),
    }
}
