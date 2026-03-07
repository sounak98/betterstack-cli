use anyhow::Result;

use crate::context::AppContext;
use crate::output::CommandOutput;
use crate::types::{CreateMonitorGroupRequest, MonitorGroupResource, UpdateMonitorGroupRequest};

#[derive(clap::Args)]
pub struct MonitorGroupsCmd {
    #[command(subcommand)]
    command: Option<MonitorGroupsSubCmd>,
}

#[derive(clap::Subcommand)]
enum MonitorGroupsSubCmd {
    /// List all monitor groups.
    List,
    /// Get details of a monitor group.
    #[command(arg_required_else_help = true)]
    Get {
        /// Monitor group ID.
        id: String,
    },
    /// Create a new monitor group.
    #[command(arg_required_else_help = true)]
    Create {
        /// Group name.
        #[arg(long)]
        name: String,
        /// Sort index for ordering.
        #[arg(long)]
        sort_index: Option<u64>,
    },
    /// Update a monitor group.
    #[command(arg_required_else_help = true)]
    Update {
        /// Monitor group ID.
        id: String,
        /// New name.
        #[arg(long)]
        name: Option<String>,
        /// Sort index for ordering.
        #[arg(long)]
        sort_index: Option<u64>,
    },
    /// Delete a monitor group.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Monitor group ID.
        id: String,
    },
}

impl MonitorGroupsCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let cmd = match &self.command {
            Some(cmd) => cmd,
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs monitor-groups", about = "Manage monitor groups.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<MonitorGroupsSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                return Ok(CommandOutput::Empty);
            }
        };
        match cmd {
            MonitorGroupsSubCmd::List => {
                let groups = ctx.uptime.list_monitor_groups().await?;
                Ok(groups_to_table(groups))
            }
            MonitorGroupsSubCmd::Get { id } => {
                let group = ctx.uptime.get_monitor_group(id).await?;
                Ok(group_to_detail(&group))
            }
            MonitorGroupsSubCmd::Create { name, sort_index } => {
                let req = CreateMonitorGroupRequest {
                    name: name.clone(),
                    sort_index: *sort_index,
                };
                let group = ctx.uptime.create_monitor_group(&req).await?;
                Ok(group_to_detail(&group))
            }
            MonitorGroupsSubCmd::Update {
                id,
                name,
                sort_index,
            } => {
                let req = UpdateMonitorGroupRequest {
                    name: name.clone(),
                    sort_index: *sort_index,
                };
                let group = ctx.uptime.update_monitor_group(id, &req).await?;
                Ok(group_to_detail(&group))
            }
            MonitorGroupsSubCmd::Delete { id } => {
                ctx.uptime.delete_monitor_group(id).await?;
                Ok(CommandOutput::Message(format!(
                    "Monitor group (ID: {id}) deleted."
                )))
            }
        }
    }
}

fn groups_to_table(groups: Vec<MonitorGroupResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Paused".to_string(),
        "Team".to_string(),
        "Created".to_string(),
    ];
    let rows = groups
        .iter()
        .map(|g| {
            let a = &g.attributes;
            vec![
                g.id.clone(),
                a.name.clone().unwrap_or_else(|| "-".to_string()),
                a.paused
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                a.team_name.clone().unwrap_or_else(|| "-".to_string()),
                a.created_at.clone().unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}

fn group_to_detail(g: &MonitorGroupResource) -> CommandOutput {
    let a = &g.attributes;
    let fields = vec![
        ("ID".to_string(), g.id.clone()),
        (
            "Name".to_string(),
            a.name.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Paused".to_string(),
            a.paused
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
        (
            "Updated".to_string(),
            a.updated_at.clone().unwrap_or_else(|| "-".to_string()),
        ),
    ];
    CommandOutput::Detail { fields }
}
