use anyhow::Result;

use crate::adapters::config::schema::SqlAuthConfig;
use crate::adapters::http::sql::SqlClient;
use crate::context::AppContext;
use crate::output::CommandOutput;
use crate::types::SourceResource;

#[derive(clap::Args)]
pub struct LogsCmd {
    #[command(subcommand)]
    command: Option<LogsSubCmd>,
}

#[derive(clap::Subcommand)]
enum LogsSubCmd {
    /// List log sources.
    Sources,
    /// Get details of a specific source.
    #[command(arg_required_else_help = true)]
    Source {
        /// Source ID.
        id: String,
    },
    /// Run a raw ClickHouse SQL query.
    #[command(arg_required_else_help = true)]
    Sql {
        /// SQL query string.
        query: String,
        /// Limit number of results.
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// Query logs with a simple filter syntax.
    #[command(arg_required_else_help = true)]
    Query {
        /// Filter expression (e.g. "level:error AND status:>=500").
        filter: String,
        /// Source ID, name, or table name (use `bs logs sources` to list).
        #[arg(long)]
        source: String,
        /// Time range (e.g. 1h, 30m, 7d).
        #[arg(long)]
        since: Option<String>,
        /// Limit number of results.
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// Live tail logs from a source (polling).
    ///
    /// Outputs pretty-printed logs by default. Use -o json for full raw JSON
    /// (useful for piping to AI tools or jq).
    #[command(arg_required_else_help = true)]
    Tail {
        /// Source ID, name, or table name (use `bs logs sources` to list).
        #[arg(long)]
        source: String,
        /// How far back to start (e.g. 5m, 1h). Defaults to 5m.
        #[arg(long, default_value = "5m")]
        since: String,
        /// Poll interval in seconds.
        #[arg(long, default_value = "2")]
        interval: u64,
    },
}

impl LogsCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let cmd = match &self.command {
            Some(cmd) => cmd,
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs logs", about = "Query and manage logs.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<LogsSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                return Ok(CommandOutput::Empty);
            }
        };
        match cmd {
            LogsSubCmd::Sources => {
                let telemetry = ctx.telemetry.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "No Telemetry API token configured. Run `bs auth init` to set one up."
                    )
                })?;
                let sources = telemetry.list_sources().await?;
                Ok(sources_to_table(sources))
            }
            LogsSubCmd::Source { id } => {
                let telemetry = ctx.telemetry.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "No Telemetry API token configured. Run `bs auth init` to set one up."
                    )
                })?;
                let source = telemetry.get_source(id).await?;
                Ok(source_to_detail(&source))
            }
            LogsSubCmd::Sql { query, limit } => {
                let sql_client = build_sql_client(ctx).await?;
                let upper = query.trim_start().to_uppercase();
                let is_select = upper.starts_with("SELECT");
                let sql = if is_select && !upper.contains("LIMIT") {
                    format!("{query} LIMIT {limit}")
                } else {
                    query.clone()
                };
                if is_select {
                    let rows = sql_client.query_json(&sql).await?;
                    Ok(json_rows_to_output(rows))
                } else {
                    let raw = sql_client.query(&sql).await?;
                    if raw.trim().is_empty() {
                        Ok(CommandOutput::Message("OK".to_string()))
                    } else {
                        Ok(CommandOutput::Message(raw))
                    }
                }
            }
            LogsSubCmd::Query {
                filter,
                source,
                since,
                limit,
            } => {
                let table = resolve_table_name(source, ctx).await?;
                let sql_client = build_sql_client(ctx).await?;
                let sql = crate::query::compile(filter, &table, *limit, since.as_deref())?;
                let rows = sql_client.query_json(&sql).await?;
                Ok(log_rows_to_output(rows, ctx.global.output_format))
            }
            LogsSubCmd::Tail {
                source,
                since,
                interval,
            } => {
                let table = resolve_table_name(source, ctx).await?;
                let sql_client = build_sql_client(ctx).await?;
                let json_output = ctx.global.output_format != crate::context::OutputFormat::Table;
                run_tail(
                    &sql_client,
                    &table,
                    since,
                    *interval,
                    ctx.global.no_color,
                    json_output,
                )
                .await
            }
        }
    }
}

/// Build the full ClickHouse remote() table reference: t{team_id}_{table_name}_logs
fn format_remote_table(team_id: u64, table_name: &str) -> String {
    format!("t{team_id}_{table_name}_logs")
}

/// Resolve a source argument to a ClickHouse remote() table reference.
/// Accepts: source ID (numeric), source name, table name, or a raw reference (t{id}_..._logs).
async fn resolve_table_name(source: &str, ctx: &AppContext) -> Result<String> {
    // If it already looks like a full remote table ref (t1234_..._logs), use as-is
    if source.starts_with('t') && source.ends_with("_logs") {
        return Ok(source.to_string());
    }

    // Try to resolve via Telemetry API
    if let Some(telemetry) = ctx.telemetry.as_ref() {
        // If numeric, try as source ID
        if source.chars().all(|c| c.is_ascii_digit())
            && let Ok(s) = telemetry.get_source(source).await
            && let (Some(team_id), Some(table)) = (s.attributes.team_id, &s.attributes.table_name)
        {
            return Ok(format_remote_table(team_id, table));
        }

        // Try matching by name or table_name
        if let Ok(sources) = telemetry.list_sources().await {
            for s in &sources {
                let name_match = s
                    .attributes
                    .name
                    .as_deref()
                    .is_some_and(|n| n.eq_ignore_ascii_case(source));
                let table_match = s
                    .attributes
                    .table_name
                    .as_deref()
                    .is_some_and(|t| t == source);
                if (name_match || table_match || s.id == source)
                    && let (Some(team_id), Some(table)) =
                        (s.attributes.team_id, &s.attributes.table_name)
                {
                    return Ok(format_remote_table(team_id, table));
                }
            }
        }
    }

    // Fall back to using the source string directly
    Ok(source.to_string())
}

async fn build_sql_client(ctx: &AppContext) -> Result<SqlClient> {
    // Check if we already have SQL credentials saved
    if let Ok(config) = ctx.config.load()
        && let Some(sql) = &config.auth.sql
        && let (Some(host), Some(username), Some(password)) =
            (&sql.host, &sql.username, &sql.password)
    {
        return Ok(SqlClient::new(host, username, password));
    }

    // Try auto-provisioning via Telemetry API
    if let Some(telemetry) = ctx.telemetry.as_ref() {
        eprintln!("Setting up SQL Query API connection...");
        match telemetry.create_sql_connection().await {
            Ok(creds) => {
                let mut config = ctx.config.load().unwrap_or_default();
                config.auth.sql = Some(SqlAuthConfig {
                    host: Some(creds.host.clone()),
                    username: Some(creds.username.clone()),
                    password: Some(creds.password.clone()),
                });
                ctx.config.save(&config)?;
                eprintln!("SQL connection created and saved.");
                return Ok(SqlClient::new(
                    &creds.host,
                    &creds.username,
                    &creds.password,
                ));
            }
            Err(_) => {
                eprintln!("Auto-setup failed (needs a global API token).");
                eprintln!("Enter SQL credentials manually (get them from your team admin).\n");
            }
        }
    } else {
        eprintln!("No Telemetry token configured. Enter SQL credentials manually.\n");
    }

    // Fall back to interactive prompt
    let host = super::prompt("SQL host (e.g. eu-nbg-2-connect.betterstackdata.com)")?;
    if host.is_empty() {
        anyhow::bail!("SQL host is required. Ask your team admin for the connection details.");
    }
    let username = super::prompt("SQL username")?;
    if username.is_empty() {
        anyhow::bail!("SQL username is required.");
    }
    let password = super::prompt_secret("SQL password")?;
    if password.is_empty() {
        anyhow::bail!("SQL password is required.");
    }

    // Save for next time
    let mut config = ctx.config.load().unwrap_or_default();
    config.auth.sql = Some(SqlAuthConfig {
        host: Some(host.clone()),
        username: Some(username.clone()),
        password: Some(password.clone()),
    });
    ctx.config.save(&config)?;
    eprintln!("SQL credentials saved.\n");

    Ok(SqlClient::new(&host, &username, &password))
}

fn sources_to_table(sources: Vec<SourceResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Platform".to_string(),
        "Table".to_string(),
        "Ingesting".to_string(),
        "Created".to_string(),
    ];

    let rows: Vec<Vec<String>> = sources
        .iter()
        .map(|s| {
            let a = &s.attributes;
            vec![
                s.id.clone(),
                a.name.clone().unwrap_or_else(|| "-".to_string()),
                a.platform.clone().unwrap_or_else(|| "-".to_string()),
                a.table_name.clone().unwrap_or_else(|| "-".to_string()),
                a.ingesting_paused
                    .map(|p| if p { "paused" } else { "active" }.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                a.created_at
                    .as_deref()
                    .and_then(|s| s.split('T').next())
                    .unwrap_or("-")
                    .to_string(),
            ]
        })
        .collect();

    CommandOutput::Table { headers, rows }
}

fn source_to_detail(s: &SourceResource) -> CommandOutput {
    let a = &s.attributes;
    let fields = vec![
        ("ID".to_string(), s.id.clone()),
        (
            "Name".to_string(),
            a.name.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Platform".to_string(),
            a.platform.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Table".to_string(),
            a.table_name.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Token".to_string(),
            a.token.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Ingesting".to_string(),
            a.ingesting_paused
                .map(|p| if p { "paused" } else { "active" }.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Retention".to_string(),
            a.retention
                .map(|r| format!("{r} days"))
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Live Tail URL".to_string(),
            a.live_tail_url.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Created".to_string(),
            a.created_at.clone().unwrap_or_else(|| "-".to_string()),
        ),
    ];
    CommandOutput::Detail { fields }
}

/// Generic table output for arbitrary SQL result rows (used by `bs logs sql`).
fn json_rows_to_output(rows: Vec<serde_json::Value>) -> CommandOutput {
    if rows.is_empty() {
        return CommandOutput::Message("No results found.".to_string());
    }

    let first = &rows[0];
    let headers: Vec<String> = if let Some(obj) = first.as_object() {
        obj.keys().cloned().collect()
    } else {
        let headers = vec!["value".to_string()];
        let table_rows = rows.iter().map(|r| vec![r.to_string()]).collect();
        return CommandOutput::Table {
            headers,
            rows: table_rows,
        };
    };

    let table_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            headers
                .iter()
                .map(|h| match row.get(h) {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(v) => v.to_string(),
                    None => "-".to_string(),
                })
                .collect()
        })
        .collect();

    CommandOutput::Table {
        headers,
        rows: table_rows,
    }
}

// ---------------------------------------------------------------------------
// Shared log row parsing (used by both query and tail)
// ---------------------------------------------------------------------------

/// Parse the `raw` JSON string from a SQL result row into a JSON object.
fn parse_raw_log(row: &serde_json::Value) -> serde_json::Value {
    row.get("raw")
        .and_then(|v| v.as_str())
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| row.clone())
}

/// Extract (dt, level, message) from a parsed log JSON object.
/// Uses top-level fields directly. Configure VRL transforms on your source
/// (`bs logs source-update <id> --vrl '...'`) to shape fields at ingestion time.
fn extract_log_fields(parsed: &serde_json::Value) -> (String, String, String) {
    let dt = parsed
        .get("dt")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let level = parsed
        .get("level")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let message = parsed
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    (dt, level, message)
}

/// Format-aware output for log query results.
/// Parses the `raw` JSON and shows extracted fields in table/csv, full parsed JSON in json mode.
fn log_rows_to_output(
    rows: Vec<serde_json::Value>,
    format: crate::context::OutputFormat,
) -> CommandOutput {
    if rows.is_empty() {
        return CommandOutput::Message("No results found.".to_string());
    }

    let parsed: Vec<serde_json::Value> = rows.iter().map(parse_raw_log).collect();

    match format {
        crate::context::OutputFormat::Json => {
            let json_str = parsed
                .iter()
                .map(|v| serde_json::to_string(v).unwrap_or_default())
                .collect::<Vec<_>>()
                .join("\n");
            CommandOutput::Raw(json_str)
        }
        crate::context::OutputFormat::Table | crate::context::OutputFormat::Csv => {
            let headers = vec!["dt".to_string(), "level".to_string(), "message".to_string()];
            let table_rows: Vec<Vec<String>> = parsed
                .iter()
                .map(|p| {
                    let (dt, level, message) = extract_log_fields(p);
                    let short_dt = if dt.len() > 19 { &dt[..19] } else { &dt };
                    vec![utc_to_local(short_dt), level, message]
                })
                .collect();
            CommandOutput::Table {
                headers,
                rows: table_rows,
            }
        }
    }
}

async fn run_tail(
    sql_client: &SqlClient,
    source: &str,
    since: &str,
    interval: u64,
    no_color: bool,
    json_output: bool,
) -> Result<CommandOutput> {
    use std::io::Write;

    eprintln!("Tailing logs from '{source}' (Ctrl+C to stop)...\n");

    let time_filter = crate::query::parse_duration_filter(since)?;
    let mut last_dt: Option<String> = None;

    // Phase 1: fetch all historical logs in the --since window (chronological order)
    let initial_sql =
        format!("SELECT dt, raw FROM remote({source}) WHERE {time_filter} ORDER BY dt ASC");
    let rows = sql_client.query_json(&initial_sql).await?;
    if rows.is_empty() {
        eprintln!("No logs found in the last {since}. Waiting for new logs...");
    } else {
        eprintln!("Showing {} logs from the last {since}:\n", rows.len());
    }
    for row in &rows {
        print_tail_row(row, no_color, json_output);
        if let Some(dt) = row.get("dt").and_then(|v| v.as_str()) {
            last_dt = Some(dt.to_string());
        }
    }
    std::io::stdout().flush()?;

    // Phase 2: poll for new logs
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!("\nStopped.");
                return Ok(CommandOutput::Empty);
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(interval)) => {}
        }

        let poll_sql = if let Some(ref dt) = last_dt {
            let escaped_dt = dt.replace('\'', "\\'");
            format!(
                "SELECT dt, raw FROM remote({source}) WHERE dt > '{escaped_dt}' ORDER BY dt ASC LIMIT 1000"
            )
        } else {
            // No logs yet, keep checking the since window
            format!(
                "SELECT dt, raw FROM remote({source}) WHERE {time_filter} ORDER BY dt ASC LIMIT 1000"
            )
        };

        match sql_client.query_json(&poll_sql).await {
            Ok(rows) => {
                for row in &rows {
                    print_tail_row(row, no_color, json_output);
                    if let Some(dt) = row.get("dt").and_then(|v| v.as_str()) {
                        last_dt = Some(dt.to_string());
                    }
                }
                std::io::stdout().flush()?;
            }
            Err(e) => {
                eprintln!("Poll error: {e}");
            }
        }
    }
}

/// Convert a UTC timestamp like "2026-03-05 21:16:40" or "2026-03-05T21:16:40" to local time.
fn utc_to_local(utc_str: &str) -> String {
    // Parse "YYYY-MM-DD HH:MM:SS" or "YYYY-MM-DDTHH:MM:SS"
    let parts: Vec<&str> = utc_str.split(&['-', ' ', 'T', ':'][..]).collect();
    if parts.len() < 6 {
        return utc_str.to_string();
    }
    let (year, month, day, hour, min, sec) = match (
        parts[0].parse::<i32>(),
        parts[1].parse::<i32>(),
        parts[2].parse::<i32>(),
        parts[3].parse::<i32>(),
        parts[4].parse::<i32>(),
        parts[5].parse::<i32>(),
    ) {
        (Ok(y), Ok(mo), Ok(d), Ok(h), Ok(mi), Ok(s)) => (y, mo, d, h, mi, s),
        _ => return utc_str.to_string(),
    };

    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        tm.tm_year = year - 1900;
        tm.tm_mon = month - 1;
        tm.tm_mday = day;
        tm.tm_hour = hour;
        tm.tm_min = min;
        tm.tm_sec = sec;
        tm.tm_isdst = -1;

        // Convert to epoch assuming UTC
        let epoch = libc::timegm(&mut tm);
        if epoch == -1 {
            return utc_str.to_string();
        }

        // Convert epoch to local time
        let local = libc::localtime(&epoch);
        if local.is_null() {
            return utc_str.to_string();
        }
        let lt = &*local;
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            lt.tm_year + 1900,
            lt.tm_mon + 1,
            lt.tm_mday,
            lt.tm_hour,
            lt.tm_min,
            lt.tm_sec
        )
    }
}

fn print_tail_row(row: &serde_json::Value, no_color: bool, json_output: bool) {
    if json_output {
        let parsed = parse_raw_log(row);
        println!(
            "{}",
            serde_json::to_string(&parsed).unwrap_or_else(|_| row.to_string())
        );
    } else {
        print_log_line(row, no_color);
    }
}

fn print_log_line(row: &serde_json::Value, no_color: bool) {
    use crate::output::color;

    let parsed = parse_raw_log(row);
    let (dt, level, message) = extract_log_fields(&parsed);

    let error = parsed.get("error").and_then(|v| v.as_str()).unwrap_or("");

    let short_dt = if dt.len() > 19 { &dt[..19] } else { &dt };
    let short_dt = utc_to_local(short_dt);

    let level_display = if no_color {
        format!("[{level}]")
    } else {
        match level.to_uppercase().as_str() {
            "ERROR" | "FATAL" | "CRITICAL" => color::red(&format!("[{level}]")),
            "WARN" | "WARNING" => format!("\x1b[33m[{level}]\x1b[0m"),
            "INFO" => color::green(&format!("[{level}]")),
            "DEBUG" | "TRACE" => color::dim(&format!("[{level}]")),
            _ => format!("[{level}]"),
        }
    };

    let dt_display = if no_color {
        short_dt.clone()
    } else {
        color::dim(&short_dt)
    };

    let content = if !error.is_empty() {
        if error.len() > 200 {
            format!("{} | {}...", message.trim(), &error[..200])
        } else {
            format!("{} | {}", message.trim(), error)
        }
    } else if !message.is_empty() {
        message.trim().to_string()
    } else {
        serde_json::to_string(&parsed).unwrap_or_default()
    };

    if level.is_empty() {
        println!("{dt_display}  {content}");
    } else {
        println!("{dt_display}  {level_display} {content}");
    }
}
