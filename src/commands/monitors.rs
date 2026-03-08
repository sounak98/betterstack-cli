use anyhow::Result;

use crate::context::{AppContext, OutputFormat};
use crate::output::CommandOutput;
use crate::output::color;
use crate::output::fmt;
use crate::types::{
    CreateMonitorRequest, MonitorFilters, MonitorResource, RegionResponseTimes,
    ResponseTimesResource, SlaResource, UpdateMonitorRequest,
};

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
    #[command(arg_required_else_help = true)]
    Get {
        /// Monitor ID.
        id: String,
    },
    /// Create a new monitor.
    #[command(arg_required_else_help = true)]
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
    #[command(arg_required_else_help = true)]
    Pause {
        /// Monitor ID.
        id: String,
    },
    /// Resume a paused monitor.
    #[command(arg_required_else_help = true)]
    Resume {
        /// Monitor ID.
        id: String,
    },
    /// Update a monitor.
    #[command(arg_required_else_help = true)]
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
    #[command(arg_required_else_help = true)]
    Delete {
        /// Monitor ID.
        id: String,
    },
    /// Show availability/SLA for a monitor.
    #[command(arg_required_else_help = true)]
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
    #[command(arg_required_else_help = true)]
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
                if ctx.global.output_format == OutputFormat::Table {
                    let sla = ctx.uptime.monitor_sla(id, None, None).await.ok();
                    let rt = ctx.uptime.monitor_response_times(id, None, None).await.ok();
                    Ok(monitor_detail_rich(&monitor, sla.as_ref(), rt.as_ref()))
                } else {
                    Ok(CommandOutput::Detail {
                        fields: build_monitor_fields(&monitor),
                    })
                }
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
                Ok(CommandOutput::Detail {
                    fields: build_monitor_fields(&monitor),
                })
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
                Ok(CommandOutput::Detail {
                    fields: build_monitor_fields(&monitor),
                })
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
                    .map(|f| fmt::duration(f as u64))
                    .unwrap_or_else(|| "-".to_string()),
                a.last_checked_at
                    .as_deref()
                    .map(fmt::relative_time)
                    .unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();

    CommandOutput::Table { headers, rows }
}

fn build_monitor_fields(m: &MonitorResource) -> Vec<(String, String)> {
    let a = &m.attributes;
    vec![
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
                .map(|f| fmt::duration(f as u64))
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
            a.last_checked_at
                .as_deref()
                .map(fmt::relative_time)
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Created".to_string(),
            a.created_at
                .as_deref()
                .map(fmt::relative_time)
                .unwrap_or_else(|| "-".to_string()),
        ),
    ]
}

fn monitor_detail_rich(
    m: &MonitorResource,
    sla: Option<&SlaResource>,
    rt: Option<&ResponseTimesResource>,
) -> CommandOutput {
    let mut out = String::new();
    let fields = build_monitor_fields(m);
    let max_label = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, value) in &fields {
        out.push_str(&format!(
            "{} {}\n",
            color::bold(&format!("{key:<max_label$}")),
            value
        ));
    }

    // Inline SLA
    if let Some(sla) = sla {
        let a = &sla.attributes;
        out.push_str(&format!("\n{}\n", color::bold("Availability (30 days)")));
        if let Some(avail) = a.availability {
            out.push_str(&format!("  Uptime       {}\n", format_availability(avail)));
        }
        if let Some(dt) = a.total_downtime {
            out.push_str(&format!("  Downtime     {}\n", fmt::duration(dt as u64)));
        }
        if let Some(n) = a.number_of_incidents {
            out.push_str(&format!("  Incidents    {n}\n"));
        }
        if let Some(longest) = a.longest_incident
            && longest > 0.0
        {
            out.push_str(&format!(
                "  Longest      {}\n",
                fmt::duration(longest as u64)
            ));
        }
    }

    // Inline response times (per-region percentiles)
    if let Some(rt) = rt
        && let Some(regions) = &rt.attributes.regions
    {
        let summaries: Vec<(String, RegionSummary)> = regions
            .iter()
            .filter_map(|r| {
                let name = r.region.as_deref().unwrap_or("unknown").to_string();
                compute_percentiles(r).map(|s| (name, s))
            })
            .collect();
        if !summaries.is_empty() {
            out.push_str(&format!("\n{}\n", color::bold("Response Times")));
            for (name, s) in &summaries {
                out.push_str(&format!(
                    "  {:<4} avg={:<8} p50={:<8} p95={:<8} p99={:<8} ({} checks)\n",
                    name,
                    format!("{:.0}ms", s.avg),
                    format!("{:.0}ms", s.p50),
                    format!("{:.0}ms", s.p95),
                    format!("{:.0}ms", s.p99),
                    s.count,
                ));
            }
        }
    }

    CommandOutput::Raw(out.trim_end().to_string())
}

fn format_availability(pct: f64) -> String {
    let text = format!("{:.4}%", pct);
    if pct >= 99.9 {
        color::green(&text)
    } else if pct >= 99.0 {
        color::yellow(&text)
    } else {
        color::red(&text)
    }
}

struct RegionSummary {
    avg: f64,
    p50: f64,
    p95: f64,
    p99: f64,
    count: usize,
}

fn compute_percentiles(region: &RegionResponseTimes) -> Option<RegionSummary> {
    let entries = region.response_times.as_ref()?;
    let mut times: Vec<f64> = entries.iter().filter_map(|e| e.response_time).collect();
    if times.is_empty() {
        return None;
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let count = times.len();
    let avg: f64 = times.iter().sum::<f64>() / count as f64;
    // Convert seconds to milliseconds
    let to_ms = 1000.0;
    Some(RegionSummary {
        avg: avg * to_ms,
        p50: percentile(&times, 50.0) * to_ms,
        p95: percentile(&times, 95.0) * to_ms,
        p99: percentile(&times, 99.0) * to_ms,
        count,
    })
}

fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (pct / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn sla_to_detail(sla: &SlaResource) -> CommandOutput {
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
                .map(|v| fmt::duration(v as u64))
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
                .map(|v| fmt::duration(v as u64))
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Avg Incident".to_string(),
            a.average_incident
                .map(|v| fmt::duration(v as u64))
                .unwrap_or_else(|| "-".to_string()),
        ),
    ];
    CommandOutput::Detail { fields }
}

fn response_times_to_table(resource: &ResponseTimesResource) -> CommandOutput {
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
                        entry
                            .at
                            .as_deref()
                            .map(fmt::relative_time)
                            .unwrap_or_else(|| "-".to_string()),
                        region_name.to_string(),
                        entry
                            .response_time
                            .map(|v| format!("{:.0}ms", v * 1000.0))
                            .unwrap_or_else(|| "-".to_string()),
                    ]);
                }
            }
        }
    }

    CommandOutput::Table { headers, rows }
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
