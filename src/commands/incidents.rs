use anyhow::Result;

use crate::context::AppContext;
use crate::context::OutputFormat;
use crate::output::CommandOutput;
use crate::output::color;
use crate::output::fmt;
use crate::types::{
    CommentResource, CreateCommentRequest, CreateIncidentRequest, EscalateIncidentRequest,
    IncidentFilters, IncidentResource, TimelineEvent,
};

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
        /// Filter by heartbeat ID.
        #[arg(long)]
        heartbeat: Option<String>,
        /// Filter incidents starting from this date (ISO 8601).
        #[arg(long)]
        from: Option<String>,
        /// Filter incidents up to this date (ISO 8601).
        #[arg(long)]
        to: Option<String>,
    },
    /// Get details of a specific incident (includes recent timeline).
    #[command(arg_required_else_help = true)]
    Get {
        /// Incident ID.
        id: String,
    },
    /// Create a new incident.
    #[command(arg_required_else_help = true)]
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
        /// Requester email or identifier.
        #[arg(long)]
        by: Option<String>,
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
        /// Send critical push notification (bypasses Do Not Disturb).
        #[arg(long)]
        critical_alert: bool,
        /// Seconds before escalating to entire team.
        #[arg(long)]
        team_wait: Option<u32>,
        /// Escalation policy ID.
        #[arg(long)]
        policy: Option<String>,
    },
    /// Acknowledge an incident.
    #[command(arg_required_else_help = true)]
    Ack {
        /// Incident ID.
        id: String,
        /// Email or identifier (defaults to config email from `bs auth init`).
        #[arg(long)]
        by: Option<String>,
    },
    /// Resolve an incident.
    #[command(arg_required_else_help = true)]
    Resolve {
        /// Incident ID.
        id: String,
        /// Email or identifier (defaults to config email from `bs auth init`).
        #[arg(long)]
        by: Option<String>,
    },
    /// Escalate an incident.
    #[command(arg_required_else_help = true)]
    Escalate {
        /// Incident ID.
        id: String,
        /// Escalation target type: User, Team, Schedule, Policy, or Organization.
        #[arg(long = "type", value_name = "TYPE")]
        escalation_type: String,
        /// User email (when --type User).
        #[arg(long)]
        user_email: Option<String>,
        /// User ID (when --type User).
        #[arg(long)]
        user_id: Option<String>,
        /// Team name (when --type Team).
        #[arg(long)]
        team_name: Option<String>,
        /// Team ID (when --type Team).
        #[arg(long)]
        team_id: Option<String>,
        /// Schedule ID (when --type Schedule).
        #[arg(long)]
        schedule_id: Option<String>,
        /// Escalation policy ID (when --type Policy).
        #[arg(long)]
        policy_id: Option<String>,
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
        /// Send critical push notification (bypasses Do Not Disturb).
        #[arg(long)]
        critical_alert: bool,
    },
    /// Delete an incident.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Incident ID.
        id: String,
    },
    /// Show the timeline for an incident.
    #[command(arg_required_else_help = true)]
    Timeline {
        /// Incident ID.
        id: String,
    },
    /// Manage incident comments.
    #[command(subcommand)]
    Comments(CommentsSubCmd),
}

#[derive(clap::Subcommand)]
enum CommentsSubCmd {
    /// List comments on an incident.
    #[command(arg_required_else_help = true)]
    List {
        /// Incident ID.
        incident_id: String,
    },
    /// Add a comment to an incident.
    #[command(arg_required_else_help = true)]
    Add {
        /// Incident ID.
        incident_id: String,
        /// Comment content (supports Markdown).
        #[arg(long)]
        content: String,
    },
    /// Delete a comment.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Incident ID.
        incident_id: String,
        /// Comment ID.
        comment_id: String,
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
                heartbeat,
                from,
                to,
            } => {
                let filters = IncidentFilters {
                    status: status.clone(),
                    monitor_id: monitor.clone(),
                    heartbeat_id: heartbeat.clone(),
                    from: from.clone(),
                    to: to.clone(),
                };
                let incidents = ctx.uptime.list_incidents(&filters).await?;
                Ok(incidents_to_table(incidents))
            }
            IncidentsSubCmd::Get { id } => {
                let incident = ctx.uptime.get_incident(id).await?;
                if ctx.global.output_format == OutputFormat::Table {
                    let timeline = ctx.uptime.incident_timeline(id).await?;
                    let comments = ctx.uptime.list_comments(id).await.unwrap_or_default();
                    Ok(incident_detail_with_timeline(
                        &incident, &timeline, &comments,
                    ))
                } else {
                    Ok(incident_to_detail(&incident))
                }
            }
            IncidentsSubCmd::Create {
                name,
                summary,
                description,
                by,
                call,
                sms,
                email,
                push,
                critical_alert,
                team_wait,
                policy,
            } => {
                let req = CreateIncidentRequest {
                    requester_email: by.clone().or_else(|| ctx.global.email.clone()),
                    name: name.clone(),
                    summary: summary.clone(),
                    description: description.clone(),
                    call: if *call { Some(true) } else { None },
                    sms: if *sms { Some(true) } else { None },
                    email: if *email { Some(true) } else { None },
                    push: if *push { Some(true) } else { None },
                    critical_alert: if *critical_alert { Some(true) } else { None },
                    team_wait: *team_wait,
                    policy_id: policy.clone(),
                };
                let incident = ctx.uptime.create_incident(&req).await?;
                Ok(incident_to_detail(&incident))
            }
            IncidentsSubCmd::Ack { id, by } => {
                let by = resolve_by(by, ctx)?;
                let incident = ctx.uptime.acknowledge_incident(id, Some(&by)).await?;
                let name = incident.attributes.name.as_deref().unwrap_or("Unknown");
                Ok(CommandOutput::Message(format!(
                    "Incident '{}' (ID: {}) acknowledged by {by}.",
                    name, incident.id
                )))
            }
            IncidentsSubCmd::Resolve { id, by } => {
                let by = resolve_by(by, ctx)?;
                let incident = ctx.uptime.resolve_incident(id, Some(&by)).await?;
                let name = incident.attributes.name.as_deref().unwrap_or("Unknown");
                Ok(CommandOutput::Message(format!(
                    "Incident '{}' (ID: {}) resolved by {by}.",
                    name, incident.id
                )))
            }
            IncidentsSubCmd::Escalate {
                id,
                escalation_type,
                user_email,
                user_id,
                team_name,
                team_id,
                schedule_id,
                policy_id,
                call,
                sms,
                email,
                push,
                critical_alert,
            } => {
                let has_channel = *call || *sms || *email || *push || *critical_alert;

                match escalation_type.as_str() {
                    "User" => {
                        if user_email.is_none() {
                            anyhow::bail!("--user-email is required when --type is User");
                        }
                    }
                    "Team" => {
                        if team_name.is_none() && team_id.is_none() {
                            anyhow::bail!(
                                "--team-name or --team-id is required when --type is Team"
                            );
                        }
                    }
                    "Schedule" => {
                        if schedule_id.is_none() {
                            anyhow::bail!("--schedule-id is required when --type is Schedule");
                        }
                    }
                    "Policy" => {
                        if policy_id.is_none() {
                            anyhow::bail!("--policy-id is required when --type is Policy");
                        }
                    }
                    "Organization" => {}
                    other => {
                        anyhow::bail!(
                            "Invalid escalation type '{other}'. Must be one of: User, Team, Schedule, Policy, Organization"
                        );
                    }
                }

                // Default to email if no channel is explicitly chosen
                let use_email = if has_channel { *email } else { true };

                let req = EscalateIncidentRequest {
                    escalation_type: escalation_type.clone(),
                    user_email: user_email.clone(),
                    user_id: user_id.clone(),
                    team_name: team_name.clone(),
                    team_id: team_id.clone(),
                    schedule_id: schedule_id.clone(),
                    policy_id: policy_id.clone(),
                    call: if *call { Some(true) } else { None },
                    sms: if *sms { Some(true) } else { None },
                    email: if use_email { Some(true) } else { None },
                    push: if *push { Some(true) } else { None },
                    critical_alert: if *critical_alert { Some(true) } else { None },
                };
                let incident = ctx.uptime.escalate_incident(id, &req).await?;
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
                if ctx.global.output_format == OutputFormat::Table {
                    Ok(CommandOutput::Raw(render_timeline(&events)))
                } else {
                    Ok(timeline_to_table(&events))
                }
            }
            IncidentsSubCmd::Comments(sub) => match sub {
                CommentsSubCmd::List { incident_id } => {
                    let comments = ctx.uptime.list_comments(incident_id).await?;
                    if ctx.global.output_format == OutputFormat::Table {
                        Ok(CommandOutput::Raw(render_comments(&comments)))
                    } else {
                        Ok(comments_to_table(&comments))
                    }
                }
                CommentsSubCmd::Add {
                    incident_id,
                    content,
                } => {
                    let by = resolve_by(&None, ctx).ok();
                    let body = if let Some(email) = &by {
                        format!("**{email}**: {content}")
                    } else {
                        content.clone()
                    };
                    let req = CreateCommentRequest { content: body };
                    let comment = ctx.uptime.create_comment(incident_id, &req).await?;
                    Ok(CommandOutput::Message(format!(
                        "Comment added (ID: {}).",
                        comment.id
                    )))
                }
                CommentsSubCmd::Delete {
                    incident_id,
                    comment_id,
                } => {
                    ctx.uptime.delete_comment(incident_id, comment_id).await?;
                    Ok(CommandOutput::Message(format!(
                        "Comment {comment_id} deleted."
                    )))
                }
            },
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
                fmt_time(a.started_at.as_deref()),
                fmt_time(a.acknowledged_at.as_deref()),
                fmt_time(a.resolved_at.as_deref()),
            ]
        })
        .collect();

    CommandOutput::Table { headers, rows }
}

fn incident_to_detail(i: &IncidentResource) -> CommandOutput {
    CommandOutput::Detail {
        fields: build_detail_fields(i),
    }
}

fn build_detail_fields(i: &IncidentResource) -> Vec<(String, String)> {
    let a = &i.attributes;
    let status = derive_status(
        a.started_at.as_deref(),
        a.acknowledged_at.as_deref(),
        a.resolved_at.as_deref(),
    );
    let mut fields = vec![
        ("ID".into(), i.id.clone()),
        ("Name".into(), s(&a.name)),
        ("Status".into(), status),
        ("URL".into(), s(&a.url)),
        ("Cause".into(), s(&a.cause)),
    ];

    if let Some(team) = &a.team_name {
        fields.push(("Team".into(), team.clone()));
    }

    fields.push(("Started".into(), s(&a.started_at)));

    if a.acknowledged_at.is_some() || a.acknowledged_by.is_some() {
        fields.push(("Acknowledged".into(), s(&a.acknowledged_at)));
        fields.push(("Acknowledged By".into(), s(&a.acknowledged_by)));
    }

    if a.resolved_at.is_some() || a.resolved_by.is_some() {
        fields.push(("Resolved".into(), s(&a.resolved_at)));
        fields.push(("Resolved By".into(), s(&a.resolved_by)));
    }

    if let Some(regions) = &a.regions
        && !regions.is_empty()
    {
        fields.push(("Regions".into(), regions.join(", ")));
    }

    if let Some(url) = &a.origin_url
        && !url.is_empty()
    {
        fields.push(("Origin URL".into(), url.clone()));
    }

    // Notification channels - compact display
    let channels: Vec<&str> = [
        a.call.filter(|&v| v).map(|_| "call"),
        a.sms.filter(|&v| v).map(|_| "sms"),
        a.email.filter(|&v| v).map(|_| "email"),
        a.push.filter(|&v| v).map(|_| "push"),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !channels.is_empty() {
        fields.push(("Notifications".into(), channels.join(", ")));
    }

    fields
}

/// Render incident detail with inline timeline and comments.
fn incident_detail_with_timeline(
    i: &IncidentResource,
    timeline: &[TimelineEvent],
    comments: &[CommentResource],
) -> CommandOutput {
    let mut out = String::new();

    // Detail fields
    let fields = build_detail_fields(i);
    let max_label = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, value) in &fields {
        out.push_str(&format!(
            "{} {}\n",
            color::bold(&format!("{key:<max_label$}")),
            value
        ));
    }

    // Timeline section
    if !timeline.is_empty() {
        out.push('\n');
        out.push_str(&format!("{}\n", color::bold("Timeline")));
        out.push_str(&render_timeline(timeline));
    }

    // Comments section
    if !comments.is_empty() {
        out.push('\n');
        out.push_str(&format!("{}\n", color::bold("Comments")));
        out.push_str(&render_comments(comments));
    }

    CommandOutput::Raw(out.trim_end().to_string())
}

fn render_timeline(events: &[TimelineEvent]) -> String {
    let mut out = String::new();
    for (idx, e) in events.iter().enumerate() {
        let a = &e.attributes;
        let item_type = a.item_type.as_deref().unwrap_or("generic");
        let time = fmt_time(a.at.as_deref());

        let content = a.data.as_ref().and_then(|d| match &d.content {
            Some(serde_json::Value::String(s)) => Some(s.clone()),
            Some(serde_json::Value::Object(obj)) => {
                obj.get("text").and_then(|t| t.as_str()).map(String::from)
            }
            _ => None,
        });

        let title = a.data.as_ref().and_then(|d| match &d.title {
            Some(serde_json::Value::String(s)) => Some(s.clone()),
            _ => None,
        });

        let icon = match item_type {
            "generic" => {
                let text = content.as_deref().unwrap_or("");
                if text.contains("started") {
                    color::red("●")
                } else if text.contains("cknowledged") {
                    color::yellow("●")
                } else if text.contains("esolved") {
                    color::green("●")
                } else if text.contains("scalat") {
                    color::yellow("▲")
                } else {
                    color::dim("●")
                }
            }
            "comment" => color::cyan("○"),
            "payload" => color::yellow("◆"),
            "response_item" => color::dim("◇"),
            "timeline_truncated" => color::dim("…"),
            _ => color::dim("●"),
        };

        let label = match item_type {
            "generic" | "generic_card" => content.clone().unwrap_or_else(|| "-".to_string()),
            "comment" => {
                let user = a.data.as_ref().and_then(|d| {
                    if let Some(serde_json::Value::Object(obj)) = &d.content {
                        obj.get("user")
                            .and_then(|u| u.get("name"))
                            .and_then(|n| n.as_str())
                            .map(String::from)
                    } else {
                        None
                    }
                });
                let text = content.as_deref().unwrap_or("");
                let snippet = if text.len() > 80 {
                    format!("{}...", &text[..77])
                } else {
                    text.to_string()
                };
                if let Some(name) = user {
                    format!("{}: {snippet}", color::bold(&name))
                } else {
                    snippet
                }
            }
            "payload" => title
                .or(content.clone())
                .unwrap_or_else(|| "External alert".to_string()),
            "response_item" => title
                .or(content.clone())
                .unwrap_or_else(|| "Response".to_string()),
            "timeline_truncated" => title.unwrap_or_else(|| "...".to_string()),
            _ => content.or(title).unwrap_or_else(|| item_type.to_string()),
        };

        let time_str = color::dim(&time);
        out.push_str(&format!("  {icon} {label}  {time_str}\n"));

        if idx < events.len() - 1 {
            out.push_str("  │\n");
        }
    }
    out
}

fn render_comments(comments: &[CommentResource]) -> String {
    let mut out = String::new();
    for (idx, c) in comments.iter().enumerate() {
        let a = &c.attributes;
        let author = a.user_email.as_deref().unwrap_or("unknown");
        let time = fmt_time(a.created_at.as_deref());
        let content = a.content.as_deref().unwrap_or("");

        out.push_str(&format!(
            "  {} {}  {}\n",
            color::bold(author),
            color::dim(&format!("#{}", c.id)),
            color::dim(&time),
        ));

        for line in content.lines() {
            out.push_str(&format!("    {line}\n"));
        }

        if idx < comments.len() - 1 {
            out.push('\n');
        }
    }
    out
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

fn s(opt: &Option<String>) -> String {
    opt.clone().unwrap_or_else(|| "-".to_string())
}

fn fmt_time(t: Option<&str>) -> String {
    match t {
        Some(ts) => fmt::relative_time(ts),
        None => "-".to_string(),
    }
}

/// Resolve the --by value: flag > config email > error.
fn resolve_by(flag: &Option<String>, ctx: &AppContext) -> Result<String> {
    if let Some(by) = flag {
        return Ok(by.clone());
    }
    if let Some(email) = &ctx.global.email {
        return Ok(email.clone());
    }
    anyhow::bail!("No --by provided and no email configured. Run `bs auth init` to set your email.")
}

fn timeline_to_table(events: &[TimelineEvent]) -> CommandOutput {
    let headers = vec!["At".to_string(), "Type".to_string(), "Content".to_string()];
    let rows = events
        .iter()
        .map(|e| {
            let a = &e.attributes;
            let content = a
                .data
                .as_ref()
                .and_then(|d| match &d.content {
                    Some(serde_json::Value::String(s)) => Some(s.clone()),
                    Some(serde_json::Value::Object(obj)) => {
                        obj.get("text").and_then(|t| t.as_str()).map(String::from)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| "-".to_string());
            vec![
                a.at.clone().unwrap_or_else(|| "-".to_string()),
                a.item_type.clone().unwrap_or_else(|| "-".to_string()),
                content,
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}

fn comments_to_table(comments: &[CommentResource]) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Author".to_string(),
        "Content".to_string(),
        "Created At".to_string(),
    ];
    let rows = comments
        .iter()
        .map(|c| {
            let a = &c.attributes;
            vec![
                c.id.clone(),
                a.user_email.clone().unwrap_or_else(|| "-".to_string()),
                a.content.clone().unwrap_or_else(|| "-".to_string()),
                a.created_at.clone().unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}
