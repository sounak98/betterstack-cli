use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::Shell;

use bs_cli::adapters::config::FileConfigStore;
use bs_cli::adapters::http::HttpClient;
use bs_cli::commands::{
    AuthCmd, HeartbeatGroupsCmd, HeartbeatsCmd, IncidentsCmd, LogsCmd, MonitorGroupsCmd,
    MonitorsCmd, OnCallCmd, PoliciesCmd, SeveritiesCmd, SourcesCmd, StatusPagesCmd,
};
use bs_cli::context::{AppContext, GlobalOptions, OutputFormat};
use bs_cli::output;

#[derive(Parser)]
#[command(name = "bs", version, about = "Fast, AI-friendly CLI for Better Stack")]
struct Cli {
    /// Output format: table, json, csv.
    #[arg(short, long, global = true, default_value = "table", env = "BS_OUTPUT")]
    output: String,

    /// Team name (for multi-team accounts).
    #[arg(long, global = true, env = "BS_TEAM")]
    team: Option<String>,

    /// Disable colored output.
    #[arg(long, global = true, env = "NO_COLOR")]
    no_color: bool,

    /// Minimal output (just IDs/statuses).
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Override Uptime API token.
    #[arg(long, global = true, env = "BETTERSTACK_UPTIME_TOKEN")]
    token: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Manage authentication.
    Auth(AuthCmd),
    /// Manage heartbeats.
    Heartbeats(HeartbeatsCmd),
    /// Manage heartbeat groups.
    HeartbeatGroups(HeartbeatGroupsCmd),
    /// Manage incidents.
    Incidents(IncidentsCmd),
    /// Query and manage logs.
    Logs(LogsCmd),
    /// Manage monitor groups.
    MonitorGroups(MonitorGroupsCmd),
    /// Manage uptime monitors.
    Monitors(MonitorsCmd),
    /// Manage on-call calendars.
    #[command(name = "oncall")]
    OnCall(OnCallCmd),
    /// Manage escalation policies.
    Policies(PoliciesCmd),
    /// Manage severities (urgency levels).
    Severities(SeveritiesCmd),
    /// Manage log sources.
    Sources(SourcesCmd),
    /// Manage status pages, sections, resources, and reports.
    StatusPages(StatusPagesCmd),
    /// Update bs to the latest version.
    Upgrade,
    /// Generate shell completions.
    Completions {
        /// Shell to generate completions for (bash, zsh, fish, powershell, elvish).
        shell: Shell,
    },
}

fn resolve_token(cli_token: Option<&str>, config_store: &FileConfigStore) -> Option<String> {
    if let Some(t) = cli_token {
        return Some(t.to_string());
    }
    if let Ok(config) = config_store.load() {
        return config.auth.uptime_token;
    }
    None
}

fn resolve_telemetry_token(config_store: &FileConfigStore) -> Option<String> {
    if let Ok(t) = std::env::var("BETTERSTACK_TELEMETRY_TOKEN")
        && !t.is_empty()
    {
        return Some(t);
    }
    if let Ok(config) = config_store.load() {
        return config.auth.telemetry_token;
    }
    None
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            Cli::command().print_help()?;
            println!();
            return Ok(());
        }
    };

    if let Command::Completions { shell } = command {
        clap_complete::generate(shell, &mut Cli::command(), "bs", &mut std::io::stdout());
        return Ok(());
    }

    let output_format: OutputFormat = cli.output.parse()?;
    let config_store = FileConfigStore::new(FileConfigStore::default_path());

    let token = resolve_token(cli.token.as_deref(), &config_store);

    // Auth and upgrade commands don't require a token
    let needs_token = !matches!(
        command,
        Command::Auth(_) | Command::Upgrade | Command::Logs(_) | Command::Sources(_)
    );

    let uptime = if needs_token {
        let token = token.ok_or_else(|| {
            anyhow::anyhow!(
                "No API token found. Run `bs auth init` or set BETTERSTACK_UPTIME_TOKEN."
            )
        })?;
        HttpClient::uptime(&token)
    } else {
        HttpClient::uptime("")
    };

    let telemetry = resolve_telemetry_token(&config_store).map(|t| HttpClient::telemetry(&t));

    let email = config_store.load().ok().and_then(|c| c.defaults.email);

    let ctx = AppContext {
        uptime,
        telemetry,
        config: config_store,
        global: GlobalOptions {
            output_format,
            email,
            team: cli.team,
            no_color: cli.no_color,
            quiet: cli.quiet,
        },
    };

    let result = match command {
        Command::Auth(cmd) => cmd.run(&ctx).await,
        Command::Heartbeats(cmd) => cmd.run(&ctx).await,
        Command::HeartbeatGroups(cmd) => cmd.run(&ctx).await,
        Command::Incidents(cmd) => cmd.run(&ctx).await,
        Command::Logs(cmd) => cmd.run(&ctx).await,
        Command::MonitorGroups(cmd) => cmd.run(&ctx).await,
        Command::Monitors(cmd) => cmd.run(&ctx).await,
        Command::OnCall(cmd) => cmd.run(&ctx).await,
        Command::Policies(cmd) => cmd.run(&ctx).await,
        Command::Severities(cmd) => cmd.run(&ctx).await,
        Command::Sources(cmd) => cmd.run(&ctx).await,
        Command::StatusPages(cmd) => cmd.run(&ctx).await,
        Command::Upgrade => bs_cli::commands::upgrade::run().await,
        Command::Completions { .. } => unreachable!(),
    };

    match result {
        Ok(cmd_output) => {
            let rendered =
                output::render(&cmd_output, ctx.global.output_format, ctx.global.no_color);
            if !rendered.is_empty() {
                println!("{rendered}");
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {e:#}");
            std::process::exit(1);
        }
    }
}
