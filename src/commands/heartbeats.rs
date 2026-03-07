use anyhow::Result;

use crate::context::AppContext;
use crate::output::CommandOutput;
use crate::types::{
    CreateHeartbeatRequest, HeartbeatFilters, HeartbeatResource, UpdateHeartbeatRequest,
};

#[derive(clap::Args)]
pub struct HeartbeatsCmd {
    #[command(subcommand)]
    command: Option<HeartbeatsSubCmd>,
}

#[derive(clap::Subcommand)]
enum HeartbeatsSubCmd {
    /// List all heartbeats.
    List {
        /// Filter by status: up, down, paused, pending.
        #[arg(long)]
        status: Option<String>,
    },
    /// Get details of a heartbeat.
    #[command(arg_required_else_help = true)]
    Get {
        /// Heartbeat ID.
        id: String,
    },
    /// Create a new heartbeat.
    #[command(arg_required_else_help = true)]
    Create {
        /// Heartbeat name.
        #[arg(long)]
        name: String,
        /// Expected check-in period in seconds (min: 30).
        #[arg(long)]
        period: u64,
        /// Grace period in seconds before alerting.
        #[arg(long)]
        grace: Option<u64>,
        /// Enable email notifications.
        #[arg(long)]
        email: bool,
        /// Enable SMS notifications.
        #[arg(long)]
        sms: bool,
        /// Enable phone call notifications.
        #[arg(long)]
        call: bool,
        /// Enable push notifications.
        #[arg(long)]
        push: bool,
        /// Heartbeat group ID.
        #[arg(long)]
        group: Option<u64>,
        /// Escalation policy ID.
        #[arg(long)]
        policy: Option<u64>,
        /// Start paused.
        #[arg(long)]
        paused: bool,
    },
    /// Update a heartbeat.
    #[command(arg_required_else_help = true)]
    Update {
        /// Heartbeat ID.
        id: String,
        /// New name.
        #[arg(long)]
        name: Option<String>,
        /// New period in seconds.
        #[arg(long)]
        period: Option<u64>,
        /// New grace period in seconds.
        #[arg(long)]
        grace: Option<u64>,
        /// Enable/disable email notifications.
        #[arg(long)]
        email: Option<bool>,
        /// Enable/disable SMS notifications.
        #[arg(long)]
        sms: Option<bool>,
        /// Enable/disable phone call notifications.
        #[arg(long)]
        call: Option<bool>,
        /// Enable/disable push notifications.
        #[arg(long)]
        push: Option<bool>,
        /// Heartbeat group ID.
        #[arg(long)]
        group: Option<u64>,
        /// Escalation policy ID.
        #[arg(long)]
        policy: Option<u64>,
        /// Pause or unpause.
        #[arg(long)]
        paused: Option<bool>,
    },
    /// Pause a heartbeat.
    #[command(arg_required_else_help = true)]
    Pause {
        /// Heartbeat ID.
        id: String,
    },
    /// Resume a paused heartbeat.
    #[command(arg_required_else_help = true)]
    Resume {
        /// Heartbeat ID.
        id: String,
    },
    /// Delete a heartbeat.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Heartbeat ID.
        id: String,
    },
    /// Show availability/SLA for a heartbeat.
    #[command(arg_required_else_help = true)]
    Availability {
        /// Heartbeat ID.
        id: String,
        /// Start date (ISO 8601).
        #[arg(long)]
        from: Option<String>,
        /// End date (ISO 8601).
        #[arg(long)]
        to: Option<String>,
    },
}

impl HeartbeatsCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let cmd = match &self.command {
            Some(cmd) => cmd,
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs heartbeats", about = "Manage heartbeats.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<HeartbeatsSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                return Ok(CommandOutput::Empty);
            }
        };
        match cmd {
            HeartbeatsSubCmd::List { status } => {
                let filters = HeartbeatFilters {
                    status: status.clone(),
                };
                let heartbeats = ctx.uptime.list_heartbeats().await?;
                let heartbeats = filter_heartbeats(heartbeats, &filters);
                Ok(heartbeats_to_table(heartbeats))
            }
            HeartbeatsSubCmd::Get { id } => {
                let hb = ctx.uptime.get_heartbeat(id).await?;
                Ok(heartbeat_to_detail(&hb))
            }
            HeartbeatsSubCmd::Create {
                name,
                period,
                grace,
                email,
                sms,
                call,
                push,
                group,
                policy,
                paused,
            } => {
                let req = CreateHeartbeatRequest {
                    name: name.clone(),
                    period: *period,
                    grace: *grace,
                    call: if *call { Some(true) } else { None },
                    sms: if *sms { Some(true) } else { None },
                    email: if *email { Some(true) } else { None },
                    push: if *push { Some(true) } else { None },
                    critical_alert: None,
                    team_wait: None,
                    heartbeat_group_id: *group,
                    sort_index: None,
                    paused: if *paused { Some(true) } else { None },
                    policy_id: *policy,
                };
                let hb = ctx.uptime.create_heartbeat(&req).await?;
                Ok(heartbeat_to_detail(&hb))
            }
            HeartbeatsSubCmd::Update {
                id,
                name,
                period,
                grace,
                email,
                sms,
                call,
                push,
                group,
                policy,
                paused,
            } => {
                let req = UpdateHeartbeatRequest {
                    name: name.clone(),
                    period: *period,
                    grace: *grace,
                    call: *call,
                    sms: *sms,
                    email: *email,
                    push: *push,
                    critical_alert: None,
                    team_wait: None,
                    heartbeat_group_id: *group,
                    sort_index: None,
                    paused: *paused,
                    policy_id: *policy,
                };
                let hb = ctx.uptime.update_heartbeat(id, &req).await?;
                Ok(heartbeat_to_detail(&hb))
            }
            HeartbeatsSubCmd::Pause { id } => {
                let req = UpdateHeartbeatRequest {
                    paused: Some(true),
                    ..default_update()
                };
                let hb = ctx.uptime.update_heartbeat(id, &req).await?;
                let name = hb.attributes.name.as_deref().unwrap_or("Unknown");
                Ok(CommandOutput::Message(format!(
                    "Heartbeat '{name}' (ID: {}) paused.",
                    hb.id
                )))
            }
            HeartbeatsSubCmd::Resume { id } => {
                let req = UpdateHeartbeatRequest {
                    paused: Some(false),
                    ..default_update()
                };
                let hb = ctx.uptime.update_heartbeat(id, &req).await?;
                let name = hb.attributes.name.as_deref().unwrap_or("Unknown");
                Ok(CommandOutput::Message(format!(
                    "Heartbeat '{name}' (ID: {}) resumed.",
                    hb.id
                )))
            }
            HeartbeatsSubCmd::Delete { id } => {
                ctx.uptime.delete_heartbeat(id).await?;
                Ok(CommandOutput::Message(format!(
                    "Heartbeat (ID: {id}) deleted."
                )))
            }
            HeartbeatsSubCmd::Availability { id, from, to } => {
                let sla = ctx
                    .uptime
                    .heartbeat_availability(id, from.as_deref(), to.as_deref())
                    .await?;
                Ok(sla_to_detail(&sla))
            }
        }
    }
}

fn filter_heartbeats(
    heartbeats: Vec<HeartbeatResource>,
    filters: &HeartbeatFilters,
) -> Vec<HeartbeatResource> {
    heartbeats
        .into_iter()
        .filter(|h| {
            if let Some(ref status) = filters.status
                && h.attributes.status.as_deref() != Some(status.as_str())
            {
                return false;
            }
            true
        })
        .collect()
}

fn heartbeats_to_table(heartbeats: Vec<HeartbeatResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Period".to_string(),
        "Grace".to_string(),
        "Status".to_string(),
        "URL".to_string(),
    ];
    let rows: Vec<Vec<String>> = heartbeats
        .iter()
        .map(|h| {
            let a = &h.attributes;
            vec![
                h.id.clone(),
                a.name.clone().unwrap_or_else(|| "-".to_string()),
                a.period
                    .map(format_seconds)
                    .unwrap_or_else(|| "-".to_string()),
                a.grace
                    .map(format_seconds)
                    .unwrap_or_else(|| "-".to_string()),
                a.status.clone().unwrap_or_else(|| "-".to_string()),
                a.url.clone().unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}

fn heartbeat_to_detail(h: &HeartbeatResource) -> CommandOutput {
    let a = &h.attributes;
    let fields = vec![
        ("ID".to_string(), h.id.clone()),
        (
            "Name".to_string(),
            a.name.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Status".to_string(),
            a.status.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "URL".to_string(),
            a.url.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Period".to_string(),
            a.period
                .map(format_seconds)
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Grace".to_string(),
            a.grace
                .map(format_seconds)
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Email".to_string(),
            a.email
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
            "Call".to_string(),
            a.call
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Push".to_string(),
            a.push
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Team".to_string(),
            a.team_name.clone().unwrap_or_else(|| "-".to_string()),
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
        ("Heartbeat ID".to_string(), sla.id.clone()),
        (
            "Availability".to_string(),
            a.availability
                .map(|v| format!("{:.4}%", v))
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Total Downtime".to_string(),
            a.total_downtime
                .map(|v| format_seconds(v as u64))
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
                .map(|v| format_seconds(v as u64))
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Avg Incident".to_string(),
            a.average_incident
                .map(|v| format_seconds(v as u64))
                .unwrap_or_else(|| "-".to_string()),
        ),
    ];
    CommandOutput::Detail { fields }
}

fn format_seconds(s: u64) -> String {
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m", s / 60)
    } else if s < 86400 {
        format!("{}h", s / 3600)
    } else {
        format!("{}d", s / 86400)
    }
}

fn default_update() -> UpdateHeartbeatRequest {
    UpdateHeartbeatRequest {
        name: None,
        period: None,
        grace: None,
        call: None,
        sms: None,
        email: None,
        push: None,
        critical_alert: None,
        team_wait: None,
        heartbeat_group_id: None,
        sort_index: None,
        paused: None,
        policy_id: None,
    }
}
