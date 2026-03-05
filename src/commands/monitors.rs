use anyhow::Result;

use crate::context::AppContext;
use crate::output::CommandOutput;
use crate::types::{CreateMonitorRequest, MonitorFilters, MonitorResource};

#[derive(clap::Args)]
pub struct MonitorsCmd {
    #[command(subcommand)]
    command: Option<MonitorsSubCmd>,
}

#[derive(clap::Subcommand)]
enum MonitorsSubCmd {
    /// List all monitors.
    List {
        /// Filter by status: up, down, paused, pending, maintenance, validating.
        #[arg(long)]
        status: Option<String>,
        /// Filter by monitor type: status, expected_status_code, keyword, ping, tcp, etc.
        #[arg(long, alias = "type")]
        monitor_type: Option<String>,
    },
    /// Get details of a specific monitor.
    Get {
        /// Monitor ID.
        id: String,
    },
    /// Create a new monitor.
    Create {
        /// URL to monitor.
        #[arg(long)]
        url: String,
        /// Display name.
        #[arg(long)]
        name: String,
        /// Monitor type: status, keyword, ping, tcp, udp, smtp, pop, imap, dns.
        #[arg(long, alias = "type", default_value = "status")]
        monitor_type: String,
        /// Check frequency in seconds.
        #[arg(long)]
        frequency: Option<u32>,
        /// Required keyword (for keyword monitors).
        #[arg(long)]
        keyword: Option<String>,
        /// Port (for tcp/udp/smtp/pop/imap monitors).
        #[arg(long)]
        port: Option<String>,
        /// Regions to check from (us, eu, as, au). Comma-separated.
        #[arg(long, value_delimiter = ',')]
        regions: Option<Vec<String>>,
        /// Enable email alerts.
        #[arg(long)]
        email: bool,
    },
    /// Pause a monitor.
    Pause {
        /// Monitor ID.
        id: String,
    },
    /// Resume a paused monitor.
    Resume {
        /// Monitor ID.
        id: String,
    },
    /// Delete a monitor.
    Delete {
        /// Monitor ID.
        id: String,
    },
}

impl MonitorsCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let cmd = match &self.command {
            Some(cmd) => cmd,
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs monitors", about = "Manage uptime monitors.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<MonitorsSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                return Ok(CommandOutput::Empty);
            }
        };
        match cmd {
            MonitorsSubCmd::List {
                status,
                monitor_type,
            } => {
                let filters = MonitorFilters {
                    status: status.clone(),
                    monitor_type: monitor_type.clone(),
                    ..Default::default()
                };
                let monitors = ctx.uptime.list_monitors(&filters).await?;

                // Client-side filtering for status and type (API only supports url/name filters)
                let monitors = filter_monitors(monitors, &filters);

                Ok(monitors_to_table(monitors))
            }
            MonitorsSubCmd::Get { id } => {
                let monitor = ctx.uptime.get_monitor(id).await?;
                Ok(monitor_to_detail(&monitor))
            }
            MonitorsSubCmd::Create {
                url,
                name,
                monitor_type,
                frequency,
                keyword,
                port,
                regions,
                email,
            } => {
                let req = CreateMonitorRequest {
                    url: url.clone(),
                    pronounceable_name: name.clone(),
                    monitor_type: monitor_type.clone(),
                    check_frequency: *frequency,
                    required_keyword: keyword.clone(),
                    port: port.clone(),
                    regions: regions.clone(),
                    email: if *email { Some(true) } else { None },
                    ..default_create_request()
                };
                let monitor = ctx.uptime.create_monitor(&req).await?;
                Ok(monitor_to_detail(&monitor))
            }
            MonitorsSubCmd::Pause { id } => {
                let monitor = ctx.uptime.pause_monitor(id).await?;
                let name = monitor
                    .attributes
                    .pronounceable_name
                    .as_deref()
                    .unwrap_or("Unknown");
                Ok(CommandOutput::Message(format!(
                    "Monitor '{}' (ID: {}) paused.",
                    name, monitor.id
                )))
            }
            MonitorsSubCmd::Resume { id } => {
                let monitor = ctx.uptime.resume_monitor(id).await?;
                let name = monitor
                    .attributes
                    .pronounceable_name
                    .as_deref()
                    .unwrap_or("Unknown");
                Ok(CommandOutput::Message(format!(
                    "Monitor '{}' (ID: {}) resumed.",
                    name, monitor.id
                )))
            }
            MonitorsSubCmd::Delete { id } => {
                ctx.uptime.delete_monitor(id).await?;
                Ok(CommandOutput::Message(format!(
                    "Monitor (ID: {}) deleted.",
                    id
                )))
            }
        }
    }
}

fn filter_monitors(
    monitors: Vec<MonitorResource>,
    filters: &MonitorFilters,
) -> Vec<MonitorResource> {
    monitors
        .into_iter()
        .filter(|m| {
            if let Some(ref status) = filters.status
                && m.attributes.status.as_deref() != Some(status.as_str())
            {
                return false;
            }
            if let Some(ref mt) = filters.monitor_type
                && m.attributes.monitor_type.as_deref() != Some(mt.as_str())
            {
                return false;
            }
            true
        })
        .collect()
}

fn monitors_to_table(monitors: Vec<MonitorResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "URL".to_string(),
        "Type".to_string(),
        "Status".to_string(),
        "Frequency".to_string(),
        "Last Checked".to_string(),
    ];

    let rows: Vec<Vec<String>> = monitors
        .iter()
        .map(|m| {
            let a = &m.attributes;
            vec![
                m.id.clone(),
                a.pronounceable_name
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
                a.url.clone().unwrap_or_else(|| "-".to_string()),
                a.monitor_type.clone().unwrap_or_else(|| "-".to_string()),
                a.status.clone().unwrap_or_else(|| "-".to_string()),
                a.check_frequency
                    .map(|f| format!("{}s", f))
                    .unwrap_or_else(|| "-".to_string()),
                a.last_checked_at.clone().unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();

    CommandOutput::Table { headers, rows }
}

fn monitor_to_detail(m: &MonitorResource) -> CommandOutput {
    let a = &m.attributes;
    let fields = vec![
        ("ID".to_string(), m.id.clone()),
        (
            "Name".to_string(),
            a.pronounceable_name
                .clone()
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "URL".to_string(),
            a.url.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Type".to_string(),
            a.monitor_type.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Status".to_string(),
            a.status.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Frequency".to_string(),
            a.check_frequency
                .map(|f| format!("{}s", f))
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Regions".to_string(),
            a.regions
                .as_ref()
                .map(|r| r.join(", "))
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "SSL Verify".to_string(),
            a.verify_ssl
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "HTTP Method".to_string(),
            a.http_method.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Last Checked".to_string(),
            a.last_checked_at.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Created".to_string(),
            a.created_at.clone().unwrap_or_else(|| "-".to_string()),
        ),
    ];
    CommandOutput::Detail { fields }
}

fn default_create_request() -> CreateMonitorRequest {
    CreateMonitorRequest {
        url: String::new(),
        pronounceable_name: String::new(),
        monitor_type: String::new(),
        check_frequency: None,
        regions: None,
        required_keyword: None,
        port: None,
        email: None,
        sms: None,
        call: None,
        verify_ssl: None,
        expected_status_codes: None,
        http_method: None,
        request_timeout: None,
        confirmation_period: None,
        recovery_period: None,
        team_name: None,
        paused: None,
    }
}
