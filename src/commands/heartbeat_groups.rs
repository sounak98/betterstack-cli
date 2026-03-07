use anyhow::Result;

use crate::context::AppContext;
use crate::output::CommandOutput;
use crate::types::{
    CreateHeartbeatGroupRequest, HeartbeatGroupResource, UpdateHeartbeatGroupRequest,
};

#[derive(clap::Args)]
pub struct HeartbeatGroupsCmd {
    #[command(subcommand)]
    command: Option<HeartbeatGroupsSubCmd>,
}

#[derive(clap::Subcommand)]
enum HeartbeatGroupsSubCmd {
    /// List all heartbeat groups.
    List,
    /// Get details of a heartbeat group.
    #[command(arg_required_else_help = true)]
    Get {
        /// Heartbeat group ID.
        id: String,
    },
    /// Create a new heartbeat group.
    #[command(arg_required_else_help = true)]
    Create {
        /// Group name.
        #[arg(long)]
        name: String,
        /// Start paused.
        #[arg(long)]
        paused: bool,
    },
    /// Update a heartbeat group.
    #[command(arg_required_else_help = true)]
    Update {
        /// Heartbeat group ID.
        id: String,
        /// New name.
        #[arg(long)]
        name: Option<String>,
        /// Pause or unpause.
        #[arg(long)]
        paused: Option<bool>,
    },
    /// Delete a heartbeat group.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Heartbeat group ID.
        id: String,
    },
}

impl HeartbeatGroupsCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let cmd = match &self.command {
            Some(cmd) => cmd,
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs heartbeat-groups", about = "Manage heartbeat groups.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<HeartbeatGroupsSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                return Ok(CommandOutput::Empty);
            }
        };
        match cmd {
            HeartbeatGroupsSubCmd::List => {
                let groups = ctx.uptime.list_heartbeat_groups().await?;
                Ok(groups_to_table(groups))
            }
            HeartbeatGroupsSubCmd::Get { id } => {
                let group = ctx.uptime.get_heartbeat_group(id).await?;
                Ok(group_to_detail(&group))
            }
            HeartbeatGroupsSubCmd::Create { name, paused } => {
                let req = CreateHeartbeatGroupRequest {
                    name: name.clone(),
                    paused: if *paused { Some(true) } else { None },
                    sort_index: None,
                };
                let group = ctx.uptime.create_heartbeat_group(&req).await?;
                Ok(group_to_detail(&group))
            }
            HeartbeatGroupsSubCmd::Update { id, name, paused } => {
                let req = UpdateHeartbeatGroupRequest {
                    name: name.clone(),
                    paused: *paused,
                    sort_index: None,
                };
                let group = ctx.uptime.update_heartbeat_group(id, &req).await?;
                Ok(group_to_detail(&group))
            }
            HeartbeatGroupsSubCmd::Delete { id } => {
                ctx.uptime.delete_heartbeat_group(id).await?;
                Ok(CommandOutput::Message(format!(
                    "Heartbeat group (ID: {id}) deleted."
                )))
            }
        }
    }
}

fn groups_to_table(groups: Vec<HeartbeatGroupResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Paused".to_string(),
        "Team".to_string(),
        "Created".to_string(),
    ];
    let rows: Vec<Vec<String>> = groups
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

fn group_to_detail(g: &HeartbeatGroupResource) -> CommandOutput {
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
