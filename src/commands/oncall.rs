use anyhow::Result;

use crate::context::AppContext;
use crate::output::CommandOutput;
use crate::types::{
    CreateOnCallEventRequest, CreateOnCallRequest, OnCallEventResource, OnCallResource,
};

#[derive(clap::Args)]
pub struct OnCallCmd {
    #[command(subcommand)]
    command: Option<OnCallSubCmd>,
}

#[derive(clap::Subcommand)]
enum OnCallSubCmd {
    /// List all on-call calendars.
    List,
    /// Get details of an on-call calendar (includes current on-call users).
    #[command(arg_required_else_help = true)]
    Get {
        /// On-call calendar ID (or "default" for the default calendar).
        id: String,
    },
    /// Show who is currently on call.
    Who,
    /// Create a new on-call calendar.
    #[command(arg_required_else_help = true)]
    Create {
        /// Calendar name.
        #[arg(long)]
        name: String,
    },
    /// Delete an on-call calendar.
    #[command(arg_required_else_help = true)]
    Delete {
        /// On-call calendar ID.
        id: String,
    },
    /// List events for an on-call calendar.
    #[command(arg_required_else_help = true)]
    Events {
        /// On-call calendar ID.
        id: String,
    },
    /// Add an event to an on-call calendar.
    #[command(name = "add-event", arg_required_else_help = true)]
    AddEvent {
        /// On-call calendar ID.
        id: String,
        /// Start time (ISO 8601).
        #[arg(long)]
        starts_at: String,
        /// End time (ISO 8601).
        #[arg(long)]
        ends_at: String,
        /// User emails (comma-separated).
        #[arg(long, value_delimiter = ',')]
        users: Vec<String>,
        /// Mark as override event.
        #[arg(long, name = "override")]
        is_override: bool,
    },
}

impl OnCallCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let cmd = match &self.command {
            Some(cmd) => cmd,
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs oncall", about = "Manage on-call calendars.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<OnCallSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                return Ok(CommandOutput::Empty);
            }
        };
        match cmd {
            OnCallSubCmd::List => {
                let calendars = ctx.uptime.list_oncalls().await?;
                Ok(oncalls_to_table(calendars))
            }
            OnCallSubCmd::Get { id } => {
                let detail = ctx.uptime.get_oncall(id).await?;
                let cal = &detail.data;
                let a = &cal.attributes;

                let mut fields = vec![
                    ("ID".to_string(), cal.id.clone()),
                    (
                        "Name".to_string(),
                        a.name.clone().unwrap_or_else(|| "-".to_string()),
                    ),
                    (
                        "Default".to_string(),
                        a.default_calendar
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    ),
                    (
                        "Team".to_string(),
                        a.team_name.clone().unwrap_or_else(|| "-".to_string()),
                    ),
                ];

                if let Some(users) = &detail.included {
                    let user_list: Vec<String> = users
                        .iter()
                        .map(|u| {
                            let name = match (&u.attributes.first_name, &u.attributes.last_name) {
                                (Some(f), Some(l)) => format!("{f} {l}"),
                                (Some(f), None) => f.clone(),
                                _ => "Unknown".to_string(),
                            };
                            let email = u.attributes.email.as_deref().unwrap_or("");
                            if email.is_empty() {
                                name
                            } else {
                                format!("{name} <{email}>")
                            }
                        })
                        .collect();
                    fields.push((
                        "On-Call Now".to_string(),
                        if user_list.is_empty() {
                            "Nobody".to_string()
                        } else {
                            user_list.join(", ")
                        },
                    ));
                }

                Ok(CommandOutput::Detail { fields })
            }
            OnCallSubCmd::Who => {
                let detail = ctx.uptime.get_oncall("default").await?;
                if let Some(users) = &detail.included {
                    if users.is_empty() {
                        return Ok(CommandOutput::Message(
                            "Nobody is currently on call.".to_string(),
                        ));
                    }
                    let headers = vec!["Name".to_string(), "Email".to_string()];
                    let rows: Vec<Vec<String>> = users
                        .iter()
                        .map(|u| {
                            let name = match (&u.attributes.first_name, &u.attributes.last_name) {
                                (Some(f), Some(l)) => format!("{f} {l}"),
                                (Some(f), None) => f.clone(),
                                _ => "-".to_string(),
                            };
                            vec![
                                name,
                                u.attributes
                                    .email
                                    .clone()
                                    .unwrap_or_else(|| "-".to_string()),
                            ]
                        })
                        .collect();
                    Ok(CommandOutput::Table { headers, rows })
                } else {
                    Ok(CommandOutput::Message(
                        "Nobody is currently on call.".to_string(),
                    ))
                }
            }
            OnCallSubCmd::Create { name } => {
                let req = CreateOnCallRequest { name: name.clone() };
                let cal = ctx.uptime.create_oncall(&req).await?;
                let name = cal.attributes.name.as_deref().unwrap_or("Unknown");
                Ok(CommandOutput::Message(format!(
                    "On-call calendar '{name}' created (ID: {}).",
                    cal.id
                )))
            }
            OnCallSubCmd::Delete { id } => {
                ctx.uptime.delete_oncall(id).await?;
                Ok(CommandOutput::Message(format!(
                    "On-call calendar (ID: {id}) deleted."
                )))
            }
            OnCallSubCmd::Events { id } => {
                let events = ctx.uptime.list_oncall_events(id).await?;
                Ok(events_to_table(events))
            }
            OnCallSubCmd::AddEvent {
                id,
                starts_at,
                ends_at,
                users,
                is_override,
            } => {
                let req = CreateOnCallEventRequest {
                    starts_at: starts_at.clone(),
                    ends_at: ends_at.clone(),
                    users: users.clone(),
                    is_override: if *is_override { Some(true) } else { None },
                };
                ctx.uptime.create_oncall_event(id, &req).await?;
                Ok(CommandOutput::Message("On-call event created.".to_string()))
            }
        }
    }
}

fn oncalls_to_table(calendars: Vec<OnCallResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Default".to_string(),
        "Team".to_string(),
    ];
    let rows: Vec<Vec<String>> = calendars
        .iter()
        .map(|c| {
            let a = &c.attributes;
            vec![
                c.id.clone(),
                a.name.clone().unwrap_or_else(|| "-".to_string()),
                a.default_calendar
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                a.team_name.clone().unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}

fn events_to_table(events: Vec<OnCallEventResource>) -> CommandOutput {
    let headers = vec![
        "Starts At".to_string(),
        "Ends At".to_string(),
        "Users".to_string(),
        "Override".to_string(),
    ];
    let rows: Vec<Vec<String>> = events
        .iter()
        .map(|e| {
            vec![
                e.starts_at.clone().unwrap_or_else(|| "-".to_string()),
                e.ends_at.clone().unwrap_or_else(|| "-".to_string()),
                e.users
                    .as_ref()
                    .map(|u| u.join(", "))
                    .unwrap_or_else(|| "-".to_string()),
                e.is_override
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}
