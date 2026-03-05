//! Simple log query language compiler.
//!
//! Translates human-friendly filters like `level:error AND status:>=500`
//! into ClickHouse SQL for the Better Stack SQL Query API.
//!
//! Syntax:
//!   field:value          -> exact match
//!   field:>=N            -> numeric comparison (>=, <=, >, <)
//!   field:"text phrase"  -> contains (LIKE)
//!   field:/pattern*      -> wildcard (LIKE)
//!   expr AND expr        -> conjunction
//!   expr OR expr         -> disjunction

use anyhow::{Result, bail};

/// Compile a simple filter expression into a ClickHouse SQL query.
///
/// `table` is the remote table name (e.g. `t123456_source_logs`).
/// `limit` is the max rows to return.
/// `since` is an optional duration string like "1h", "30m", "7d".
pub fn compile(filter: &str, table: &str, limit: u32, since: Option<&str>) -> Result<String> {
    let where_clause = parse_filter(filter)?;
    let time_filter = since
        .map(parse_duration_filter)
        .transpose()?
        .unwrap_or_default();

    let mut conditions = Vec::new();
    if !time_filter.is_empty() {
        conditions.push(time_filter);
    }
    if !where_clause.is_empty() {
        conditions.push(where_clause);
    }

    let where_str = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    Ok(format!(
        "SELECT dt, raw FROM remote({table}){where_str} ORDER BY dt DESC LIMIT {limit}"
    ))
}

fn parse_duration_filter(since: &str) -> Result<String> {
    let since = since.trim();
    if since.is_empty() {
        return Ok(String::new());
    }

    let (num_str, unit) = if let Some(n) = since.strip_suffix('d') {
        (n, "DAY")
    } else if let Some(n) = since.strip_suffix('h') {
        (n, "HOUR")
    } else if let Some(n) = since.strip_suffix('m') {
        (n, "MINUTE")
    } else if let Some(n) = since.strip_suffix('s') {
        (n, "SECOND")
    } else {
        bail!("Invalid duration '{since}'. Use format like 1h, 30m, 7d, 60s");
    };

    let num: u64 = num_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid number in duration '{since}'"))?;

    Ok(format!("dt >= now() - INTERVAL {num} {unit}"))
}

fn parse_filter(input: &str) -> Result<String> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(String::new());
    }

    // Split on AND/OR while preserving quoted strings
    let tokens = tokenize(input)?;
    let mut parts = Vec::new();
    let mut current_op = None;

    for token in &tokens {
        match token.as_str() {
            "AND" | "and" => current_op = Some("AND"),
            "OR" | "or" => current_op = Some("OR"),
            _ => {
                let condition = compile_condition(token)?;
                if let Some(op) = current_op {
                    if let Some(last) = parts.last_mut() {
                        *last = format!("({} {op} {condition})", last);
                    }
                    current_op = None;
                } else {
                    parts.push(condition);
                }
            }
        }
    }

    Ok(parts.join(" AND "))
}

fn tokenize(input: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut current = String::new();

    while let Some(&c) = chars.peek() {
        if c == '"' {
            // Quoted string - include quotes in the token
            current.push(chars.next().unwrap());
            while let Some(&c) = chars.peek() {
                current.push(chars.next().unwrap());
                if c == '"' {
                    break;
                }
            }
        } else if c == ' ' {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            chars.next();
        } else {
            current.push(chars.next().unwrap());
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

fn compile_condition(token: &str) -> Result<String> {
    let Some(colon_pos) = token.find(':') else {
        // No colon - treat as a raw text search
        let escaped = escape_sql(token);
        return Ok(format!("raw LIKE '%{escaped}%'"));
    };

    let field = &token[..colon_pos];
    let value = &token[colon_pos + 1..];

    if field.is_empty() {
        bail!("Empty field name in filter: '{token}'");
    }

    // Check for comparison operators
    if let Some(rest) = value.strip_prefix(">=") {
        let escaped = escape_sql(rest);
        return Ok(format!(
            "toFloat64OrNull(JSONExtractString(raw, '{field}')) >= {escaped}"
        ));
    }
    if let Some(rest) = value.strip_prefix("<=") {
        let escaped = escape_sql(rest);
        return Ok(format!(
            "toFloat64OrNull(JSONExtractString(raw, '{field}')) <= {escaped}"
        ));
    }
    if let Some(rest) = value.strip_prefix('>') {
        let escaped = escape_sql(rest);
        return Ok(format!(
            "toFloat64OrNull(JSONExtractString(raw, '{field}')) > {escaped}"
        ));
    }
    if let Some(rest) = value.strip_prefix('<') {
        let escaped = escape_sql(rest);
        return Ok(format!(
            "toFloat64OrNull(JSONExtractString(raw, '{field}')) < {escaped}"
        ));
    }

    // Quoted value - contains/LIKE search
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        let inner = &value[1..value.len() - 1];
        let escaped = escape_sql(inner);
        return Ok(format!(
            "JSONExtractString(raw, '{field}') LIKE '%{escaped}%'"
        ));
    }

    // Wildcard pattern
    if value.contains('*') {
        let escaped = escape_sql(&value.replace('*', "%"));
        return Ok(format!(
            "JSONExtractString(raw, '{field}') LIKE '{escaped}'"
        ));
    }

    // Exact match
    let escaped = escape_sql(value);
    Ok(format!("JSONExtractString(raw, '{field}') = '{escaped}'"))
}

fn escape_sql(s: &str) -> String {
    s.replace('\'', "\\'").replace('\\', "\\\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_equality() {
        let sql = compile("level:error", "t123_logs", 100, None).unwrap();
        assert_eq!(
            sql,
            "SELECT dt, raw FROM remote(t123_logs) WHERE JSONExtractString(raw, 'level') = 'error' ORDER BY dt DESC LIMIT 100"
        );
    }

    #[test]
    fn numeric_comparison() {
        let sql = compile("status:>=500", "t123_logs", 50, None).unwrap();
        assert!(sql.contains("toFloat64OrNull(JSONExtractString(raw, 'status')) >= 500"));
    }

    #[test]
    fn quoted_contains() {
        let sql = compile("message:\"connection timeout\"", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("LIKE '%connection timeout%'"));
    }

    #[test]
    fn wildcard_pattern() {
        let sql = compile("path:/api/*", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("LIKE '/api/%'"));
    }

    #[test]
    fn and_conjunction() {
        let sql = compile("level:error AND status:>=500", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("AND"));
        assert!(sql.contains("level"));
        assert!(sql.contains("status"));
    }

    #[test]
    fn or_disjunction() {
        let sql = compile("level:error OR level:warn", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("OR"));
    }

    #[test]
    fn with_since_duration() {
        let sql = compile("level:error", "t123_logs", 100, Some("1h")).unwrap();
        assert!(sql.contains("dt >= now() - INTERVAL 1 HOUR"));
    }

    #[test]
    fn empty_filter_with_since() {
        let sql = compile("", "t123_logs", 100, Some("30m")).unwrap();
        assert!(sql.contains("dt >= now() - INTERVAL 30 MINUTE"));
        assert!(!sql.contains("AND AND"));
    }

    #[test]
    fn raw_text_search() {
        let sql = compile("error", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("raw LIKE '%error%'"));
    }
}
