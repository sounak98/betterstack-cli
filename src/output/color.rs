//! Minimal ANSI color helpers. No dependencies needed.

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";

pub fn bold(s: &str) -> String {
    format!("{BOLD}{s}{RESET}")
}

pub fn green(s: &str) -> String {
    format!("{GREEN}{s}{RESET}")
}

pub fn red(s: &str) -> String {
    format!("{RED}{s}{RESET}")
}

pub fn dim(s: &str) -> String {
    format!("{DIM}{s}{RESET}")
}

pub fn status(s: &str) -> String {
    match s {
        "up" => format!("{GREEN}{BOLD}{s}{RESET}"),
        "down" => format!("{RED}{BOLD}{s}{RESET}"),
        "paused" => format!("{YELLOW}{s}{RESET}"),
        "validating" | "pending" => format!("{CYAN}{s}{RESET}"),
        "maintenance" => format!("{DIM}{s}{RESET}"),
        _ => s.to_string(),
    }
}
