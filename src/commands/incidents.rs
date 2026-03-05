use anyhow::Result;

use crate::context::AppContext;
use crate::output::CommandOutput;
use crate::types::{CreateIncidentRequest, IncidentFilters, IncidentResource, TimelineEvent};

#[derive(clap::Args)]
pub struct IncidentsCmd {
    #[command(subcommand)]
    command: Option<IncidentsSubCmd>,
}

#[derive(clap::Subcommand)]
enum IncidentsSubCmd {
    /// List all incidents.
    List {
        /// Filter by status: started, acknowledged, resolved.
        #[arg(long)]
        status: Option<String>,
        /// Filter by monitor ID.
        #[arg(long)]
        monitor: Option<String>,
        /// Filter incidents starting from this date (ISO 8601).
        #[arg(long)]
        from: Option<String>,
        /// Filter incidents up to this date (ISO 8601).
        #[arg(long)]
        to: Option<String>,
    },
    /// Get details of a specific incident.
    Get {
        /// Incident ID.
        id: String,
    },
    /// Create a new incident.
    Create {
        /// Incident name.
        #[arg(long)]
        name: Option<String>,
        /// Short summary.
        #[arg(long)]
        summary: Option<String>,
        /// Detailed description.
        #[arg(long)]
        description: Option<String>,
        /// Send call alerts.
        #[arg(long)]
        call: bool,
        /// Send SMS alerts.
        #[arg(long)]
        sms: bool,
        /// Send email alerts.
        #[arg(long)]
        email: bool,
        /// Send push notifications.
        #[arg(long)]
        push: bool,
    },
    /// Acknowledge an incident.
    Ack {
        /// Incident ID.
        id: String,
    },
    /// Resolve an incident.
    Resolve {
        /// Incident ID.
        id: String,
    },
    /// Escalate an incident.
    Escalate {
        /// Incident ID.
        id: String,
    },
    /// Delete an incident.
    Delete {
        /// Incident ID.
        id: String,
    },
    /// Show the timeline for an incident.
    Timeline {
        /// Incident ID.
        id: String,
    },
}

impl IncidentsCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let cmd = match &self.command {
            Some(cmd) => cmd,
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs incidents", about = "Manage incidents.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<IncidentsSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                return Ok(CommandOutput::Empty);
            }
        };
        match cmd {
            IncidentsSubCmd::List {
                status,
                monitor,
                from,
                to,
            } => {
                let filters = IncidentFilters {
                    status: status.clone(),
                    monitor_id: monitor.clone(),
                    from: from.clone(),
                    to: to.clone(),
                };
                let incidents = ctx.uptime.list_incidents(&filters).await?;
                Ok(incidents_to_table(incidents))
            }
            IncidentsSubCmd::Get { id } => {
                let incident = ctx.uptime.get_incident(id).await?;
                Ok(incident_to_detail(&incident))
            }
            IncidentsSubCmd::Create {
                name,
                summary,
                description,
                call,
                sms,
                email,
                push,
            } => {
                let req = CreateIncidentRequest {
                    requester_email: None,
                    name: name.clone(),
                    summary: summary.clone(),
                    description: description.clone(),
                    call: if *call { Some(true) } else { None },
                    sms: if *sms { Some(true) } else { None },
                    email: if *email { Some(true) } else { None },
                    push: if *push { Some(true) } else { None },
                };
                let incident = ctx.uptime.create_incident(&req).await?;
                Ok(incident_to_detail(&incident))
            }
            IncidentsSubCmd::Ack { id } => {
                let incident = ctx.uptime.acknowledge_incident(id).await?;
                let name = incident.attributes.name.as_deref().unwrap_or("Unknown");
                Ok(CommandOutput::Message(format!(
                    "Incident '{}' (ID: {}) acknowledged.",
                    name, incident.id
                )))
            }
            IncidentsSubCmd::Resolve { id } => {
                let incident = ctx.uptime.resolve_incident(id).await?;
                let name = incident.attributes.name.as_deref().unwrap_or("Unknown");
                Ok(CommandOutput::Message(format!(
                    "Incident '{}' (ID: {}) resolved.",
                    name, incident.id
                )))
            }
            IncidentsSubCmd::Escalate { id } => {
                let incident = ctx.uptime.escalate_incident(id).await?;
                let name = incident.attributes.name.as_deref().unwrap_or("Unknown");
                Ok(CommandOutput::Message(format!(
                    "Incident '{}' (ID: {}) escalated.",
                    name, incident.id
                )))
            }
            IncidentsSubCmd::Delete { id } => {
                ctx.uptime.delete_incident(id).await?;
                Ok(CommandOutput::Message(format!(
                    "Incident (ID: {}) deleted.",
                    id
                )))
            }
            IncidentsSubCmd::Timeline { id } => {
                let events = ctx.uptime.incident_timeline(id).await?;
                Ok(timeline_to_table(events))
            }
        }
    }
}

fn incidents_to_table(incidents: Vec<IncidentResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Status".to_string(),
        "Cause".to_string(),
        "Started".to_string(),
        "Acknowledged".to_string(),
        "Resolved".to_string(),
    ];

    let rows: Vec<Vec<String>> = incidents
        .iter()
        .map(|i| {
            let a = &i.attributes;
            let status = derive_status(
                a.started_at.as_deref(),
                a.acknowledged_at.as_deref(),
                a.resolved_at.as_deref(),
            );
            vec![
                i.id.clone(),
                a.name.clone().unwrap_or_else(|| "-".to_string()),
                status,
                a.cause.clone().unwrap_or_else(|| "-".to_string()),
                a.started_at.clone().unwrap_or_else(|| "-".to_string()),
                a.acknowledged_at.clone().unwrap_or_else(|| "-".to_string()),
                a.resolved_at.clone().unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();

    CommandOutput::Table { headers, rows }
}

fn incident_to_detail(i: &IncidentResource) -> CommandOutput {
    let a = &i.attributes;
    let status = derive_status(
        a.started_at.as_deref(),
        a.acknowledged_at.as_deref(),
        a.resolved_at.as_deref(),
    );
    let fields = vec![
        ("ID".to_string(), i.id.clone()),
        (
            "Name".to_string(),
            a.name.clone().unwrap_or_else(|| "-".to_string()),
        ),
        ("Status".to_string(), status),
        (
            "URL".to_string(),
            a.url.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Cause".to_string(),
            a.cause.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Started".to_string(),
            a.started_at.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Acknowledged".to_string(),
            a.acknowledged_at.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Acknowledged By".to_string(),
            a.acknowledged_by.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Resolved".to_string(),
            a.resolved_at.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Resolved By".to_string(),
            a.resolved_by.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Call".to_string(),
            a.call
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "SMS".to_string(),
            a.sms
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Email".to_string(),
            a.email
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Push".to_string(),
            a.push
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
    ];
    CommandOutput::Detail { fields }
}

fn timeline_to_table(events: Vec<TimelineEvent>) -> CommandOutput {
    let headers = vec![
        "Time".to_string(),
        "Event".to_string(),
        "Duration".to_string(),
        "Regions".to_string(),
    ];

    let rows: Vec<Vec<String>> = events
        .iter()
        .map(|e| {
            let a = &e.attributes;
            vec![
                a.started_at.clone().unwrap_or_else(|| "-".to_string()),
                a.event_type.clone().unwrap_or_else(|| "-".to_string()),
                a.duration
                    .map(|d| format!("{:.1}s", d))
                    .unwrap_or_else(|| "-".to_string()),
                a.regions
                    .as_ref()
                    .map(|r| r.join(", "))
                    .unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();

    CommandOutput::Table { headers, rows }
}

/// Derive a human-readable incident status from timestamps.
fn derive_status(
    started_at: Option<&str>,
    acknowledged_at: Option<&str>,
    resolved_at: Option<&str>,
) -> String {
    if resolved_at.is_some() {
        "resolved".to_string()
    } else if acknowledged_at.is_some() {
        "acknowledged".to_string()
    } else if started_at.is_some() {
        "started".to_string()
    } else {
        "-".to_string()
    }
}
