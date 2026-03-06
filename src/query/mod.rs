//! Query language compiler matching Better Stack's Live Tail syntax.
//!
//! Translates Better Stack query language into ClickHouse SQL
//! for the SQL Query API.
//!
//! Syntax (matches <https://betterstack.com/docs/logs/using-logtail/live-tail-query-language/>):
//!
//!   Simple queries (fulltext search):
//!     hello                -> search across all fields
//!     "hello world"        -> quoted fulltext search
//!
//!   Compound queries (field operator value):
//!     level = ERROR        -> equals
//!     level != DEBUG       -> not equals
//!     message : "timeout"  -> contains
//!     message !: "timeout" -> not contains
//!     message =~ /pat/     -> regex match
//!     message !~ /pat/     -> regex not match
//!     status >= 500        -> numeric comparison (>=, <=, >, <)
//!
//!   Conjunctions:
//!     expr AND expr        -> conjunction (implicit when omitted)
//!     expr OR expr         -> disjunction
//!     (expr OR expr) AND expr -> grouping with parentheses

use anyhow::{Result, bail};

/// Compile a filter expression into a ClickHouse SQL query.
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

pub fn parse_duration_filter(since: &str) -> Result<String> {
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

pub fn parse_filter(input: &str) -> Result<String> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(String::new());
    }
    let tokens = tokenize(input)?;
    let mut pos = 0;
    let result = parse_expr(&tokens, &mut pos)?;
    if pos < tokens.len() {
        bail!("Unexpected token '{}' at position {pos}", tokens[pos].raw());
    }
    Ok(result)
}

// -- Token types --

#[derive(Debug, Clone)]
enum Token {
    Ident(String),
    Quoted(String),
    Regex(String, bool), // pattern, case_insensitive
    Op(OpKind),
    And,
    Or,
    LParen,
    RParen,
}

impl Token {
    fn raw(&self) -> String {
        match self {
            Token::Ident(s) => s.clone(),
            Token::Quoted(s) => format!("\"{s}\""),
            Token::Regex(p, i) => {
                if *i {
                    format!("/{p}/i")
                } else {
                    format!("/{p}/")
                }
            }
            Token::Op(op) => op.as_str().to_string(),
            Token::And => "AND".to_string(),
            Token::Or => "OR".to_string(),
            Token::LParen => "(".to_string(),
            Token::RParen => ")".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum OpKind {
    Eq,          // =
    Neq,         // !=
    Contains,    // :
    NotContains, // !:
    RegexMatch,  // =~
    RegexNot,    // !~
    Gte,         // >=
    Lte,         // <=
    Gt,          // >
    Lt,          // <
}

impl OpKind {
    fn as_str(self) -> &'static str {
        match self {
            OpKind::Eq => "=",
            OpKind::Neq => "!=",
            OpKind::Contains => ":",
            OpKind::NotContains => "!:",
            OpKind::RegexMatch => "=~",
            OpKind::RegexNot => "!~",
            OpKind::Gte => ">=",
            OpKind::Lte => "<=",
            OpKind::Gt => ">",
            OpKind::Lt => "<",
        }
    }
}

// -- Tokenizer --

fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Skip whitespace
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        // Quoted string
        if chars[i] == '"' {
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != '"' {
                s.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1; // skip closing quote
            }
            tokens.push(Token::Quoted(s));
            continue;
        }

        // Regex literal /pattern/ or /pattern/i
        if chars[i] == '/' {
            i += 1;
            let mut pat = String::new();
            while i < chars.len() && chars[i] != '/' {
                pat.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1; // skip closing /
            }
            let case_insensitive = i < chars.len() && chars[i] == 'i';
            if case_insensitive {
                i += 1;
            }
            tokens.push(Token::Regex(pat, case_insensitive));
            continue;
        }

        // Parentheses
        if chars[i] == '(' {
            tokens.push(Token::LParen);
            i += 1;
            continue;
        }
        if chars[i] == ')' {
            tokens.push(Token::RParen);
            i += 1;
            continue;
        }

        // Two-char operators: !=, !:, !~, =~, >=, <=
        if i + 1 < chars.len() {
            let two = format!("{}{}", chars[i], chars[i + 1]);
            let op = match two.as_str() {
                "!=" => Some(OpKind::Neq),
                "!:" => Some(OpKind::NotContains),
                "!~" => Some(OpKind::RegexNot),
                "=~" => Some(OpKind::RegexMatch),
                ">=" => Some(OpKind::Gte),
                "<=" => Some(OpKind::Lte),
                _ => None,
            };
            if let Some(op) = op {
                tokens.push(Token::Op(op));
                i += 2;
                continue;
            }
        }

        // Single-char operators: =, :, >, <
        match chars[i] {
            '=' => {
                tokens.push(Token::Op(OpKind::Eq));
                i += 1;
                continue;
            }
            ':' => {
                tokens.push(Token::Op(OpKind::Contains));
                i += 1;
                continue;
            }
            '>' => {
                tokens.push(Token::Op(OpKind::Gt));
                i += 1;
                continue;
            }
            '<' => {
                tokens.push(Token::Op(OpKind::Lt));
                i += 1;
                continue;
            }
            _ => {}
        }

        // Identifier (field name, bare value, AND, OR)
        let mut ident = String::new();
        while i < chars.len()
            && !chars[i].is_whitespace()
            && !matches!(
                chars[i],
                '=' | '!' | ':' | '>' | '<' | '(' | ')' | '"' | '/' | '~'
            )
        {
            ident.push(chars[i]);
            i += 1;
        }

        if !ident.is_empty() {
            match ident.as_str() {
                "AND" | "and" => tokens.push(Token::And),
                "OR" | "or" => tokens.push(Token::Or),
                _ => tokens.push(Token::Ident(ident)),
            }
        }
    }

    Ok(tokens)
}

// -- Parser --

/// Parse a full expression: terms joined by AND/OR (implicit AND).
fn parse_expr(tokens: &[Token], pos: &mut usize) -> Result<String> {
    let mut left = parse_term(tokens, pos)?;

    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::And => {
                *pos += 1;
                let right = parse_term(tokens, pos)?;
                left = format!("({left} AND {right})");
            }
            Token::Or => {
                *pos += 1;
                let right = parse_term(tokens, pos)?;
                left = format!("({left} OR {right})");
            }
            Token::RParen => break,
            // Implicit AND: next token starts a new term
            _ => {
                let right = parse_term(tokens, pos)?;
                left = format!("({left} AND {right})");
            }
        }
    }

    Ok(left)
}

/// Parse a single term: a condition, simple query, or parenthesized group.
fn parse_term(tokens: &[Token], pos: &mut usize) -> Result<String> {
    if *pos >= tokens.len() {
        bail!("Unexpected end of query");
    }

    // Parenthesized group
    if matches!(tokens[*pos], Token::LParen) {
        *pos += 1;
        let inner = parse_expr(tokens, pos)?;
        if *pos < tokens.len() && matches!(tokens[*pos], Token::RParen) {
            *pos += 1;
        } else {
            bail!("Missing closing parenthesis");
        }
        return Ok(inner);
    }

    // Quoted string at top level -> fulltext search
    if let Token::Quoted(s) = &tokens[*pos] {
        let escaped = escape_sql(s);
        *pos += 1;
        return Ok(format!("raw LIKE '%{escaped}%'"));
    }

    // Identifier: could be field name (followed by operator) or simple text search
    if let Token::Ident(ident) = &tokens[*pos] {
        let ident = ident.clone();

        // Look ahead for an operator
        if *pos + 1 < tokens.len()
            && let Token::Op(op) = &tokens[*pos + 1]
        {
            let op = *op;
            *pos += 2; // consume field and operator
            let value = parse_value(tokens, pos)?;
            return compile_condition(&ident, op, &value);
        }

        // No operator follows: simple fulltext search
        *pos += 1;
        let escaped = escape_sql(&ident);
        return Ok(format!("raw LIKE '%{escaped}%'"));
    }

    bail!("Unexpected token: {}", tokens[*pos].raw());
}

/// Parse the value part of a condition.
fn parse_value(tokens: &[Token], pos: &mut usize) -> Result<Value> {
    if *pos >= tokens.len() {
        bail!("Expected a value after operator");
    }
    let val = match &tokens[*pos] {
        Token::Ident(s) => Value::Plain(s.clone()),
        Token::Quoted(s) => Value::Quoted(s.clone()),
        Token::Regex(pat, ci) => Value::Regex(pat.clone(), *ci),
        other => bail!("Expected a value, got: {}", other.raw()),
    };
    *pos += 1;
    Ok(val)
}

enum Value {
    Plain(String),
    Quoted(String),
    Regex(String, bool),
}

// -- SQL generation --

fn json_extract(field: &str) -> String {
    // Handle escaped dots: `service\.name` -> literal dot in field name
    // Split on unescaped dots for nested fields
    let mut parts = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = field.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '.' {
            current.push('.');
            i += 2;
        } else if chars[i] == '.' {
            parts.push(std::mem::take(&mut current));
            i += 1;
        } else {
            current.push(chars[i]);
            i += 1;
        }
    }
    parts.push(current);

    let args = parts
        .iter()
        .map(|p| format!("'{}'", escape_sql(p)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("JSONExtractString(raw, {args})")
}

fn compile_condition(field: &str, op: OpKind, value: &Value) -> Result<String> {
    // fulltext pseudo-column
    if field == "fulltext" {
        let text = match value {
            Value::Plain(s) => s.clone(),
            Value::Quoted(s) => s.clone(),
            Value::Regex(p, _) => p.clone(),
        };
        let escaped = escape_sql(&text);
        return match op {
            OpKind::Eq => Ok(format!("raw LIKE '%{escaped}%'")),
            OpKind::Neq => Ok(format!("raw NOT LIKE '%{escaped}%'")),
            OpKind::Contains => Ok(format!("raw LIKE '%{escaped}%'")),
            OpKind::NotContains => Ok(format!("raw NOT LIKE '%{escaped}%'")),
            OpKind::RegexMatch => {
                if let Value::Regex(pat, ci) = value {
                    let escaped_pat = escape_sql(pat);
                    if *ci {
                        Ok(format!("match(lower(raw), lower('{escaped_pat}'))"))
                    } else {
                        Ok(format!("match(raw, '{escaped_pat}')"))
                    }
                } else {
                    Ok(format!("match(raw, '{escaped}')"))
                }
            }
            OpKind::RegexNot => {
                if let Value::Regex(pat, ci) = value {
                    let escaped_pat = escape_sql(pat);
                    if *ci {
                        Ok(format!("NOT match(lower(raw), lower('{escaped_pat}'))"))
                    } else {
                        Ok(format!("NOT match(raw, '{escaped_pat}')"))
                    }
                } else {
                    Ok(format!("NOT match(raw, '{escaped}')"))
                }
            }
            _ => bail!("Operator {} not supported for fulltext", op.as_str()),
        };
    }

    let extract = json_extract(field);

    match op {
        OpKind::Eq => {
            let escaped = value_to_sql(value);
            Ok(format!("{extract} = '{escaped}'"))
        }
        OpKind::Neq => {
            let escaped = value_to_sql(value);
            Ok(format!("{extract} != '{escaped}'"))
        }
        OpKind::Contains => {
            let escaped = value_to_sql(value);
            Ok(format!("{extract} LIKE '%{escaped}%'"))
        }
        OpKind::NotContains => {
            let escaped = value_to_sql(value);
            Ok(format!("{extract} NOT LIKE '%{escaped}%'"))
        }
        OpKind::RegexMatch => {
            let (pat, ci) = regex_parts(value)?;
            let escaped_pat = escape_sql(&pat);
            if ci {
                Ok(format!("match(lower({extract}), lower('{escaped_pat}'))"))
            } else {
                Ok(format!("match({extract}, '{escaped_pat}')"))
            }
        }
        OpKind::RegexNot => {
            let (pat, ci) = regex_parts(value)?;
            let escaped_pat = escape_sql(&pat);
            if ci {
                Ok(format!(
                    "NOT match(lower({extract}), lower('{escaped_pat}'))"
                ))
            } else {
                Ok(format!("NOT match({extract}, '{escaped_pat}')"))
            }
        }
        OpKind::Gte => {
            let escaped = value_to_sql(value);
            Ok(format!("toFloat64OrNull({extract}) >= {escaped}"))
        }
        OpKind::Lte => {
            let escaped = value_to_sql(value);
            Ok(format!("toFloat64OrNull({extract}) <= {escaped}"))
        }
        OpKind::Gt => {
            let escaped = value_to_sql(value);
            Ok(format!("toFloat64OrNull({extract}) > {escaped}"))
        }
        OpKind::Lt => {
            let escaped = value_to_sql(value);
            Ok(format!("toFloat64OrNull({extract}) < {escaped}"))
        }
    }
}

fn value_to_sql(value: &Value) -> String {
    match value {
        Value::Plain(s) => escape_sql(s),
        Value::Quoted(s) => escape_sql(s),
        Value::Regex(p, _) => escape_sql(p),
    }
}

fn regex_parts(value: &Value) -> Result<(String, bool)> {
    match value {
        Value::Regex(pat, ci) => Ok((pat.clone(), *ci)),
        Value::Plain(s) => Ok((s.clone(), false)),
        Value::Quoted(s) => Ok((s.clone(), false)),
    }
}

fn escape_sql(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_fulltext() {
        let sql = compile("error", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("raw LIKE '%error%'"));
    }

    #[test]
    fn quoted_fulltext() {
        let sql = compile("\"connection timeout\"", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("raw LIKE '%connection timeout%'"));
    }

    #[test]
    fn field_equals() {
        let sql = compile("level = ERROR", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("JSONExtractString(raw, 'level') = 'ERROR'"));
    }

    #[test]
    fn field_not_equals() {
        let sql = compile("level != DEBUG", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("JSONExtractString(raw, 'level') != 'DEBUG'"));
    }

    #[test]
    fn field_contains() {
        let sql = compile("message : \"timeout\"", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("LIKE '%timeout%'"));
    }

    #[test]
    fn field_not_contains() {
        let sql = compile("message !: \"debug\"", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("NOT LIKE '%debug%'"));
    }

    #[test]
    fn regex_match() {
        let sql = compile("message =~ /Failed (GET|POST)/", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("match("));
        assert!(sql.contains("Failed (GET|POST)"));
    }

    #[test]
    fn regex_case_insensitive() {
        let sql = compile("platform =~ /kubernetes/i", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("match(lower("));
    }

    #[test]
    fn numeric_gte() {
        let sql = compile("status >= 500", "t123_logs", 50, None).unwrap();
        assert!(sql.contains("toFloat64OrNull(JSONExtractString(raw, 'status')) >= 500"));
    }

    #[test]
    fn and_conjunction() {
        let sql = compile("level = ERROR AND status >= 500", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("AND"));
    }

    #[test]
    fn or_conjunction() {
        let sql = compile("level = ERROR OR level = WARN", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("OR"));
    }

    #[test]
    fn implicit_and() {
        let sql = compile("level = ERROR status >= 500", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("AND"));
    }

    #[test]
    fn parentheses() {
        let sql = compile(
            "(level = ERROR OR level = WARN) AND status >= 500",
            "t123_logs",
            100,
            None,
        )
        .unwrap();
        assert!(sql.contains("OR"));
        assert!(sql.contains("AND"));
    }

    #[test]
    fn with_since_duration() {
        let sql = compile("level = ERROR", "t123_logs", 100, Some("1h")).unwrap();
        assert!(sql.contains("dt >= now() - INTERVAL 1 HOUR"));
    }

    #[test]
    fn empty_filter_with_since() {
        let sql = compile("", "t123_logs", 100, Some("30m")).unwrap();
        assert!(sql.contains("dt >= now() - INTERVAL 30 MINUTE"));
        assert!(!sql.contains("AND AND"));
    }

    #[test]
    fn no_spaces_around_operator() {
        let sql = compile("level=ERROR", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("JSONExtractString(raw, 'level') = 'ERROR'"));
    }

    #[test]
    fn colon_contains_no_spaces() {
        let sql = compile("message:timeout", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("LIKE '%timeout%'"));
    }

    #[test]
    fn fulltext_column() {
        let sql = compile("fulltext : \"Record Not Found\"", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("raw LIKE '%Record Not Found%'"));
    }

    #[test]
    fn nested_field() {
        let sql = compile("message_json.level = ERROR", "t123_logs", 100, None).unwrap();
        assert!(sql.contains("JSONExtractString(raw, 'message_json', 'level')"));
    }
}
