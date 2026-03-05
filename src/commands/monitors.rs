use anyhow::Result;

use crate::context::AppContext;
use crate::output::CommandOutput;
use crate::types::{CreateMonitorRequest, MonitorFilters, MonitorResource, UpdateMonitorRequest};

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
    /// Update a monitor.
    Update {
        /// Monitor ID.
        id: String,
        /// New URL to monitor.
        #[arg(long)]
        url: Option<String>,
        /// New display name.
        #[arg(long)]
        name: Option<String>,
        /// Check frequency in seconds.
        #[arg(long)]
        frequency: Option<u32>,
        /// HTTP method (GET, POST, HEAD, etc.).
        #[arg(long)]
        http_method: Option<String>,
        /// Request timeout in seconds.
        #[arg(long)]
        timeout: Option<u32>,
        /// Confirmation period in seconds.
        #[arg(long)]
        confirmation_period: Option<u32>,
        /// Recovery period in seconds.
        #[arg(long)]
        recovery_period: Option<u32>,
        /// Enable/disable SSL verification.
        #[arg(long)]
        verify_ssl: Option<bool>,
    },
    /// Delete a monitor.
    Delete {
        /// Monitor ID.
        id: String,
    },
    /// Show availability/SLA for a monitor.
    Availability {
        /// Monitor ID.
        id: String,
        /// Start date (ISO 8601).
        #[arg(long)]
        from: Option<String>,
        /// End date (ISO 8601).
        #[arg(long)]
        to: Option<String>,
    },
    /// Show response times for a monitor.
    ResponseTimes {
        /// Monitor ID.
        id: String,
        /// Start date (ISO 8601).
        #[arg(long)]
        from: Option<String>,
        /// End date (ISO 8601).
        #[arg(long)]
        to: Option<String>,
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
            MonitorsSubCmd::Update {
                id,
                url,
                name,
                frequency,
                http_method,
                timeout,
                confirmation_period,
                recovery_period,
                verify_ssl,
            } => {
                let req = UpdateMonitorRequest {
                    url: url.clone(),
                    pronounceable_name: name.clone(),
                    monitor_type: None,
                    check_frequency: *frequency,
                    regions: None,
                    required_keyword: None,
                    port: None,
                    email: None,
                    sms: None,
                    call: None,
                    verify_ssl: *verify_ssl,
                    expected_status_codes: None,
                    http_method: http_method.clone(),
                    request_timeout: *timeout,
                    confirmation_period: *confirmation_period,
                    recovery_period: *recovery_period,
                    paused: None,
                };
                let monitor = ctx.uptime.update_monitor(id, &req).await?;
                Ok(monitor_to_detail(&monitor))
            }
            MonitorsSubCmd::Delete { id } => {
                ctx.uptime.delete_monitor(id).await?;
                Ok(CommandOutput::Message(format!(
                    "Monitor (ID: {}) deleted.",
                    id
                )))
            }
            MonitorsSubCmd::Availability { id, from, to } => {
                let sla = ctx
                    .uptime
                    .monitor_sla(id, from.as_deref(), to.as_deref())
                    .await?;
                Ok(sla_to_detail(&sla))
            }
            MonitorsSubCmd::ResponseTimes { id, from, to } => {
                let resource = ctx
                    .uptime
                    .monitor_response_times(id, from.as_deref(), to.as_deref())
                    .await?;
                Ok(response_times_to_table(&resource))
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

fn sla_to_detail(sla: &crate::types::SlaResource) -> CommandOutput {
    let a = &sla.attributes;
    let fields = vec![
        ("Monitor ID".to_string(), sla.id.clone()),
        (
            "Availability".to_string(),
            a.availability
                .map(|v| format!("{:.4}%", v))
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Total Downtime".to_string(),
            a.total_downtime
                .map(format_duration)
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Incidents".to_string(),
            a.number_of_incidents
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Longest Incident".to_string(),
            a.longest_incident
                .map(format_duration)
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Avg Incident".to_string(),
            a.average_incident
                .map(format_duration)
                .unwrap_or_else(|| "-".to_string()),
        ),
    ];
    CommandOutput::Detail { fields }
}

fn response_times_to_table(resource: &crate::types::ResponseTimesResource) -> CommandOutput {
    let headers = vec![
        "At".to_string(),
        "Region".to_string(),
        "Response Time".to_string(),
    ];

    let mut rows: Vec<Vec<String>> = Vec::new();
    if let Some(regions) = &resource.attributes.regions {
        for region_data in regions {
            let region_name = region_data.region.as_deref().unwrap_or("-");
            if let Some(entries) = &region_data.response_times {
                for entry in entries {
                    rows.push(vec![
                        entry.at.clone().unwrap_or_else(|| "-".to_string()),
                        region_name.to_string(),
                        entry
                            .response_time
                            .map(|v| format!("{:.3}s", v))
                            .unwrap_or_else(|| "-".to_string()),
                    ]);
                }
            }
        }
    }

    CommandOutput::Table { headers, rows }
}

fn format_duration(seconds: f64) -> String {
    if seconds < 60.0 {
        format!("{:.0}s", seconds)
    } else if seconds < 3600.0 {
        format!("{:.1}m", seconds / 60.0)
    } else if seconds < 86400.0 {
        format!("{:.1}h", seconds / 3600.0)
    } else {
        format!("{:.1}d", seconds / 86400.0)
    }
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
