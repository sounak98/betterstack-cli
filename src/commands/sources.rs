use anyhow::Result;

use crate::context::AppContext;
use crate::output::CommandOutput;
use crate::types::SourceResource;
use crate::types::source::SourceUpdate;

#[derive(clap::Args)]
pub struct SourcesCmd {
    #[command(subcommand)]
    command: Option<SourcesSubCmd>,
}

#[derive(clap::Subcommand)]
enum SourcesSubCmd {
    /// List all log sources.
    List,
    /// Get details of a specific source.
    #[command(arg_required_else_help = true)]
    Get {
        /// Source ID.
        id: String,
    },
    /// Update a source.
    #[command(arg_required_else_help = true)]
    Update {
        /// Source ID.
        id: String,
        /// New name.
        #[arg(long)]
        name: Option<String>,
        /// VRL transformation code.
        #[arg(long)]
        vrl: Option<String>,
        /// Live tail display pattern (e.g. "{level} {message}").
        #[arg(long)]
        live_tail_pattern: Option<String>,
        /// Log retention in days.
        #[arg(long)]
        logs_retention: Option<u32>,
        /// Metrics retention in days.
        #[arg(long)]
        metrics_retention: Option<u32>,
        /// Pause ingestion.
        #[arg(long, conflicts_with = "unpause")]
        pause: bool,
        /// Resume ingestion.
        #[arg(long, conflicts_with = "pause")]
        unpause: bool,
    },
    /// Delete a source.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Source ID.
        id: String,
    },
}

impl SourcesCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let cmd = match &self.command {
            Some(cmd) => cmd,
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs sources", about = "Manage log sources.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<SourcesSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                return Ok(CommandOutput::Empty);
            }
        };

        let telemetry = ctx.telemetry.as_ref().ok_or_else(|| {
            anyhow::anyhow!("No Telemetry API token configured. Run `bs auth init` to set one up.")
        })?;

        match cmd {
            SourcesSubCmd::List => {
                let sources = telemetry.list_sources().await?;
                Ok(sources_to_table(sources))
            }
            SourcesSubCmd::Get { id } => {
                let source = telemetry.get_source(id).await?;
                Ok(source_to_detail(&source))
            }
            SourcesSubCmd::Update {
                id,
                name,
                vrl,
                live_tail_pattern,
                logs_retention,
                metrics_retention,
                pause,
                unpause,
            } => {
                let ingesting_paused = if *pause {
                    Some(true)
                } else if *unpause {
                    Some(false)
                } else {
                    None
                };

                let update = SourceUpdate {
                    name: name.clone(),
                    vrl_transformation: vrl.clone(),
                    live_tail_pattern: live_tail_pattern.clone(),
                    logs_retention: *logs_retention,
                    metrics_retention: *metrics_retention,
                    ingesting_paused,
                };

                let source = telemetry.update_source(id, &update).await?;
                Ok(source_to_detail(&source))
            }
            SourcesSubCmd::Delete { id } => {
                telemetry.delete_source(id).await?;
                Ok(CommandOutput::Message(format!("Source {id} deleted.")))
            }
        }
    }
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
    let mut fields = vec![
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
            "Logs retention".to_string(),
            a.logs_retention
                .map(|r| format!("{r} days"))
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Metrics retention".to_string(),
            a.metrics_retention
                .map(|r| format!("{r} days"))
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Live tail pattern".to_string(),
            a.live_tail_pattern
                .clone()
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Live tail URL".to_string(),
            a.live_tail_url.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Created".to_string(),
            a.created_at.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Updated".to_string(),
            a.updated_at.clone().unwrap_or_else(|| "-".to_string()),
        ),
    ];

    if let Some(vrl) = &a.vrl_transformation
        && !vrl.is_empty()
    {
        fields.push(("VRL transformation".to_string(), vrl.clone()));
    }

    CommandOutput::Detail { fields }
}
