use anyhow::Result;

use crate::context::AppContext;
use crate::output::CommandOutput;
use crate::types::{CreateSeverityRequest, SeverityResource, UpdateSeverityRequest};

#[derive(clap::Args)]
pub struct SeveritiesCmd {
    #[command(subcommand)]
    command: Option<SeveritiesSubCmd>,
}

#[derive(clap::Subcommand)]
enum SeveritiesSubCmd {
    /// List all severities.
    List,
    /// Get details of a severity.
    #[command(arg_required_else_help = true)]
    Get {
        /// Severity ID.
        id: String,
    },
    /// Create a new severity.
    #[command(arg_required_else_help = true)]
    Create {
        /// Severity name (e.g. "P1 Critical", "P2 Warning").
        #[arg(long)]
        name: String,
        /// Disable email notifications (enabled by default).
        #[arg(long)]
        no_email: bool,
        /// Enable SMS notifications.
        #[arg(long)]
        sms: bool,
        /// Enable phone call notifications.
        #[arg(long)]
        call: bool,
        /// Enable push notifications.
        #[arg(long)]
        push: bool,
        /// Send critical alerts (ignores Do Not Disturb).
        #[arg(long)]
        critical_alert: bool,
    },
    /// Update a severity (fetches current values, applies your changes).
    #[command(arg_required_else_help = true)]
    Update {
        /// Severity ID.
        id: String,
        /// New name.
        #[arg(long)]
        name: Option<String>,
        /// Toggle email notifications on.
        #[arg(long, overrides_with = "no_email")]
        email: bool,
        /// Toggle email notifications off.
        #[arg(long)]
        no_email: bool,
        /// Toggle SMS notifications on.
        #[arg(long, overrides_with = "no_sms")]
        sms: bool,
        /// Toggle SMS notifications off.
        #[arg(long)]
        no_sms: bool,
        /// Toggle phone call notifications on.
        #[arg(long, overrides_with = "no_call")]
        call: bool,
        /// Toggle phone call notifications off.
        #[arg(long)]
        no_call: bool,
        /// Toggle push notifications on.
        #[arg(long, overrides_with = "no_push")]
        push: bool,
        /// Toggle push notifications off.
        #[arg(long)]
        no_push: bool,
        /// Toggle critical alerts on.
        #[arg(long, overrides_with = "no_critical_alert")]
        critical_alert: bool,
        /// Toggle critical alerts off.
        #[arg(long)]
        no_critical_alert: bool,
    },
    /// Delete a severity.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Severity ID.
        id: String,
    },
}

impl SeveritiesCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let cmd = match &self.command {
            Some(cmd) => cmd,
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs severities", about = "Manage severities.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<SeveritiesSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                return Ok(CommandOutput::Empty);
            }
        };
        match cmd {
            SeveritiesSubCmd::List => {
                let severities = ctx.uptime.list_severities().await?;
                Ok(severities_to_table(severities))
            }
            SeveritiesSubCmd::Get { id } => {
                let sev = ctx.uptime.get_severity(id).await?;
                Ok(severity_to_detail(&sev))
            }
            SeveritiesSubCmd::Create {
                name,
                no_email,
                sms,
                call,
                push,
                critical_alert,
            } => {
                let req = CreateSeverityRequest {
                    name: name.clone(),
                    email: !*no_email,
                    sms: *sms,
                    call: *call,
                    push: *push,
                    critical_alert: if *critical_alert { Some(true) } else { None },
                };
                let sev = ctx.uptime.create_severity(&req).await?;
                Ok(severity_to_detail(&sev))
            }
            SeveritiesSubCmd::Update {
                id,
                name,
                email,
                no_email,
                sms,
                no_sms,
                call,
                no_call,
                push,
                no_push,
                critical_alert,
                no_critical_alert,
            } => {
                let current = ctx.uptime.get_severity(id).await?;
                let ca = &current.attributes;

                let req = UpdateSeverityRequest {
                    name: Some(
                        name.clone()
                            .unwrap_or_else(|| ca.name.clone().unwrap_or_default()),
                    ),
                    email: Some(toggle(*email, *no_email, ca.email)),
                    sms: Some(toggle(*sms, *no_sms, ca.sms)),
                    call: Some(toggle(*call, *no_call, ca.call)),
                    push: Some(toggle(*push, *no_push, ca.push)),
                    critical_alert: Some(toggle(
                        *critical_alert,
                        *no_critical_alert,
                        ca.critical_alert,
                    )),
                };
                let sev = ctx.uptime.update_severity(id, &req).await?;
                Ok(severity_to_detail(&sev))
            }
            SeveritiesSubCmd::Delete { id } => {
                ctx.uptime.delete_severity(id).await?;
                Ok(CommandOutput::Message(format!(
                    "Severity (ID: {id}) deleted."
                )))
            }
        }
    }
}

fn severities_to_table(severities: Vec<SeverityResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Email".to_string(),
        "SMS".to_string(),
        "Call".to_string(),
        "Push".to_string(),
        "Critical".to_string(),
    ];
    let rows: Vec<Vec<String>> = severities
        .iter()
        .map(|s| {
            let a = &s.attributes;
            vec![
                s.id.clone(),
                a.name.clone().unwrap_or_else(|| "-".to_string()),
                bool_display(a.email),
                bool_display(a.sms),
                bool_display(a.call),
                bool_display(a.push),
                bool_display(a.critical_alert),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}

fn severity_to_detail(s: &SeverityResource) -> CommandOutput {
    let a = &s.attributes;
    let fields = vec![
        ("ID".to_string(), s.id.clone()),
        (
            "Name".to_string(),
            a.name.clone().unwrap_or_else(|| "-".to_string()),
        ),
        ("Email".to_string(), bool_display(a.email)),
        ("SMS".to_string(), bool_display(a.sms)),
        ("Call".to_string(), bool_display(a.call)),
        ("Push".to_string(), bool_display(a.push)),
        ("Critical Alert".to_string(), bool_display(a.critical_alert)),
        (
            "Team".to_string(),
            a.team_name.clone().unwrap_or_else(|| "-".to_string()),
        ),
    ];
    CommandOutput::Detail { fields }
}

fn bool_display(v: Option<bool>) -> String {
    v.map(|b| if b { "yes" } else { "no" })
        .unwrap_or("-")
        .to_string()
}

/// Resolve a toggle: --flag sets true, --no-flag sets false, neither keeps current.
fn toggle(on: bool, off: bool, current: Option<bool>) -> bool {
    if on {
        true
    } else if off {
        false
    } else {
        current.unwrap_or(false)
    }
}
