use anyhow::Result;

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
    Source {
        /// Source ID.
        id: String,
    },
    /// Run a raw ClickHouse SQL query.
    Sql {
        /// SQL query string.
        query: String,
        /// Limit number of results.
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// Query logs with a simple filter syntax.
    Query {
        /// Filter expression (e.g. "level:error AND status:>=500").
        filter: String,
        /// Source name or table name.
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
    Tail {
        /// Source name or table name.
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
                let sql_client = build_sql_client(ctx)?;
                let sql = if query.to_uppercase().contains("LIMIT") {
                    query.clone()
                } else {
                    format!("{query} LIMIT {limit}")
                };
                let rows = sql_client.query_json(&sql).await?;
                Ok(json_rows_to_output(rows))
            }
            LogsSubCmd::Query {
                filter,
                source,
                since,
                limit,
            } => {
                let sql_client = build_sql_client(ctx)?;
                let sql = crate::query::compile(filter, source, *limit, since.as_deref())?;
                let rows = sql_client.query_json(&sql).await?;
                Ok(json_rows_to_output(rows))
            }
            LogsSubCmd::Tail {
                source,
                since,
                interval,
            } => {
                let sql_client = build_sql_client(ctx)?;
                run_tail(&sql_client, source, since, *interval, ctx.global.no_color).await
            }
        }
    }
}

fn build_sql_client(ctx: &AppContext) -> Result<SqlClient> {
    let config = ctx.config.load().map_err(|_| {
        anyhow::anyhow!("No config found. Run `bs auth init` to configure SQL credentials.")
    })?;
    let sql_config = config.auth.sql.ok_or_else(|| {
        anyhow::anyhow!(
            "No SQL credentials configured. Run `bs auth init` to set up SQL Query API access."
        )
    })?;
    let username = sql_config
        .username
        .ok_or_else(|| anyhow::anyhow!("SQL username not set in config."))?;
    let password = sql_config
        .password
        .ok_or_else(|| anyhow::anyhow!("SQL password not set in config."))?;
    let region = sql_config.region.unwrap_or_else(|| "eu-nbg-2".to_string());

    Ok(SqlClient::new(&region, &username, &password))
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
                a.created_at.clone().unwrap_or_else(|| "-".to_string()),
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

fn json_rows_to_output(rows: Vec<serde_json::Value>) -> CommandOutput {
    if rows.is_empty() {
        return CommandOutput::Message("No results found.".to_string());
    }

    // Extract column names from the first row
    let first = &rows[0];
    let headers: Vec<String> = if let Some(obj) = first.as_object() {
        obj.keys().cloned().collect()
    } else {
        // Non-object rows: show as single-column table
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

async fn run_tail(
    sql_client: &SqlClient,
    source: &str,
    since: &str,
    interval: u64,
    no_color: bool,
) -> Result<CommandOutput> {
    use std::io::Write;

    // Parse the initial "since" to get a starting point
    let initial_sql = crate::query::compile("", source, 50, Some(since))?;

    let mut last_dt: Option<String> = None;

    // Print initial batch
    let rows = sql_client.query_json(&initial_sql).await?;
    for row in &rows {
        print_log_line(row, no_color);
        if let Some(dt) = row.get("dt").and_then(|v| v.as_str()) {
            last_dt = Some(dt.to_string());
        }
    }
    std::io::stdout().flush()?;

    // Poll loop
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

        let poll_sql = if let Some(ref dt) = last_dt {
            let escaped_dt = dt.replace('\'', "\\'");
            format!(
                "SELECT dt, raw FROM remote({source}) WHERE dt > '{escaped_dt}' ORDER BY dt ASC LIMIT 100"
            )
        } else {
            crate::query::compile("", source, 100, Some(since))?
        };

        match sql_client.query_json(&poll_sql).await {
            Ok(rows) => {
                for row in &rows {
                    print_log_line(row, no_color);
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

fn print_log_line(row: &serde_json::Value, _no_color: bool) {
    let dt = row.get("dt").and_then(|v| v.as_str()).unwrap_or("-");
    let raw = row.get("raw").and_then(|v| v.as_str()).unwrap_or("");

    if raw.is_empty() {
        // Print the whole row as JSON
        println!("{dt}  {row}");
    } else {
        println!("{dt}  {raw}");
    }
}
