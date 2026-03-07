//! Minimal ANSI color helpers. No dependencies needed.

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";

// Background colors
const BG_RED: &str = "\x1b[41m";
const BG_GREEN: &str = "\x1b[42m";
const BG_YELLOW: &str = "\x1b[43m";
const BG_BRIGHT_BLACK: &str = "\x1b[100m";
const FG_BLACK: &str = "\x1b[30m";
const FG_WHITE: &str = "\x1b[97m";

pub fn bold(s: &str) -> String {
    format!("{BOLD}{s}{RESET}")
}

pub fn green(s: &str) -> String {
    format!("{GREEN}{s}{RESET}")
}

pub fn red(s: &str) -> String {
    format!("{RED}{s}{RESET}")
}

pub fn yellow(s: &str) -> String {
    format!("{YELLOW}{s}{RESET}")
}

pub fn cyan(s: &str) -> String {
    format!("{CYAN}{s}{RESET}")
}

pub fn dim(s: &str) -> String {
    format!("{DIM}{s}{RESET}")
}

/// Renders a fixed-width level badge with background color.
pub fn level_badge(level: &str) -> String {
    let upper = level.to_uppercase();
    let label = match upper.as_str() {
        "WARNING" => "WARN",
        _ => &upper,
    };
    let padded = format!(" {label:<5}");
    match upper.as_str() {
        "ERROR" | "FATAL" | "CRITICAL" => {
            format!("{BG_RED}{FG_WHITE}{BOLD}{padded}{RESET}")
        }
        "WARN" | "WARNING" => {
            format!("{BG_YELLOW}{FG_BLACK}{BOLD}{padded}{RESET}")
        }
        "INFO" => {
            format!("{BG_GREEN}{FG_BLACK}{BOLD}{padded}{RESET}")
        }
        "DEBUG" | "TRACE" => {
            format!("{BG_BRIGHT_BLACK}{FG_WHITE}{padded}{RESET}")
        }
        _ => {
            format!("{BG_BRIGHT_BLACK}{FG_WHITE}{padded}{RESET}")
        }
    }
}

pub fn status(s: &str) -> String {
    match s {
        "up" => format!("{GREEN}{BOLD}{s}{RESET}"),
        "down" => format!("{RED}{BOLD}{s}{RESET}"),
        "paused" => format!("{YELLOW}{s}{RESET}"),
        "validating" | "pending" => format!("{CYAN}{s}{RESET}"),
        "maintenance" => format!("{DIM}{s}{RESET}"),
        "started" => format!("{RED}{BOLD}{s}{RESET}"),
        "acknowledged" => format!("{YELLOW}{s}{RESET}"),
        "resolved" => format!("{GREEN}{s}{RESET}"),
        // Status page aggregate states
        "operational" => format!("{GREEN}{s}{RESET}"),
        "downtime" | "degraded" | "disrupted" => format!("{RED}{BOLD}{s}{RESET}"),
        _ => s.to_string(),
    }
}
