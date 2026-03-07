use anyhow::Result;

use crate::context::AppContext;
use crate::output::CommandOutput;
use crate::types::{CreatePolicyRequest, PolicyResource, PolicyStep, UpdatePolicyRequest};

#[derive(clap::Args)]
pub struct PoliciesCmd {
    #[command(subcommand)]
    command: Option<PoliciesSubCmd>,
}

#[derive(clap::Subcommand)]
enum PoliciesSubCmd {
    /// List all escalation policies.
    List,
    /// Get details of an escalation policy.
    #[command(arg_required_else_help = true)]
    Get {
        /// Policy ID.
        id: String,
    },
    /// Create an escalation policy.
    #[command(arg_required_else_help = true)]
    Create {
        /// Policy name.
        #[arg(long)]
        name: String,
        /// Steps as JSON array (e.g. '[{"type":"escalation","wait_before":0,"urgency_id":1,"step_members":["current_on_call"]}]').
        #[arg(long)]
        steps: String,
        /// Number of times to repeat the policy.
        #[arg(long)]
        repeat_count: Option<u64>,
        /// Seconds between repetitions.
        #[arg(long)]
        repeat_delay: Option<u64>,
    },
    /// Update an escalation policy.
    #[command(arg_required_else_help = true)]
    Update {
        /// Policy ID.
        id: String,
        /// New policy name.
        #[arg(long)]
        name: Option<String>,
        /// Steps as JSON array.
        #[arg(long)]
        steps: Option<String>,
        /// Number of times to repeat the policy.
        #[arg(long)]
        repeat_count: Option<u64>,
        /// Seconds between repetitions.
        #[arg(long)]
        repeat_delay: Option<u64>,
    },
    /// Delete an escalation policy.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Policy ID.
        id: String,
    },
}

impl PoliciesCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let cmd = match &self.command {
            Some(cmd) => cmd,
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs policies", about = "Manage escalation policies.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<PoliciesSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                return Ok(CommandOutput::Empty);
            }
        };
        match cmd {
            PoliciesSubCmd::List => {
                let policies = ctx.uptime.list_policies().await?;
                Ok(policies_to_table(policies))
            }
            PoliciesSubCmd::Get { id } => {
                let policy = ctx.uptime.get_policy(id).await?;
                Ok(policy_to_detail(&policy))
            }
            PoliciesSubCmd::Create {
                name,
                steps,
                repeat_count,
                repeat_delay,
            } => {
                let parsed_steps: Vec<PolicyStep> = serde_json::from_str(steps)
                    .map_err(|e| anyhow::anyhow!("Invalid steps JSON: {e}"))?;
                let req = CreatePolicyRequest {
                    name: name.clone(),
                    steps: parsed_steps,
                    repeat_count: *repeat_count,
                    repeat_delay: *repeat_delay,
                    policy_group_id: None,
                };
                let policy = ctx.uptime.create_policy(&req).await?;
                Ok(policy_to_detail(&policy))
            }
            PoliciesSubCmd::Update {
                id,
                name,
                steps,
                repeat_count,
                repeat_delay,
            } => {
                let parsed_steps = steps
                    .as_ref()
                    .map(|s| {
                        serde_json::from_str::<Vec<PolicyStep>>(s)
                            .map_err(|e| anyhow::anyhow!("Invalid steps JSON: {e}"))
                    })
                    .transpose()?;
                let req = UpdatePolicyRequest {
                    name: name.clone(),
                    steps: parsed_steps,
                    repeat_count: *repeat_count,
                    repeat_delay: *repeat_delay,
                    policy_group_id: None,
                };
                let policy = ctx.uptime.update_policy(id, &req).await?;
                Ok(policy_to_detail(&policy))
            }
            PoliciesSubCmd::Delete { id } => {
                ctx.uptime.delete_policy(id).await?;
                Ok(CommandOutput::Message(format!(
                    "Escalation policy (ID: {id}) deleted."
                )))
            }
        }
    }
}

fn policies_to_table(policies: Vec<PolicyResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Steps".to_string(),
        "Repeat".to_string(),
        "Team".to_string(),
    ];
    let rows: Vec<Vec<String>> = policies
        .iter()
        .map(|p| {
            let a = &p.attributes;
            let step_count = a.steps.as_ref().map(|s| s.len()).unwrap_or(0);
            let repeat = match (a.repeat_count, a.repeat_delay) {
                (Some(c), Some(d)) if c > 0 => format!("{c}x every {}s", d),
                _ => "-".to_string(),
            };
            vec![
                p.id.clone(),
                a.name.clone().unwrap_or_else(|| "-".to_string()),
                format!("{step_count}"),
                repeat,
                a.team_name.clone().unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}

fn policy_to_detail(p: &PolicyResource) -> CommandOutput {
    let a = &p.attributes;
    let mut fields = vec![
        ("ID".to_string(), p.id.clone()),
        (
            "Name".to_string(),
            a.name.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Repeat Count".to_string(),
            a.repeat_count
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Repeat Delay".to_string(),
            a.repeat_delay
                .map(|v| format!("{v}s"))
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Team".to_string(),
            a.team_name.clone().unwrap_or_else(|| "-".to_string()),
        ),
    ];

    if let Some(steps) = &a.steps {
        for (i, step) in steps.iter().enumerate() {
            let step_type = step.step_type.as_deref().unwrap_or("unknown");
            let summary = match step_type {
                "escalation" => {
                    let wait = step.wait_before.unwrap_or(0);
                    let members = step
                        .step_members
                        .as_ref()
                        .map(|m| format!("{} member(s)", m.len()))
                        .unwrap_or_else(|| "no members".to_string());
                    format!("Escalation (wait: {wait}s, {members})")
                }
                "time_branching" => {
                    let tz = step.timezone.as_deref().unwrap_or("?");
                    let days = step
                        .days
                        .as_ref()
                        .map(|d| d.join(","))
                        .unwrap_or_else(|| "?".to_string());
                    format!("Time Branch ({tz}, {days})")
                }
                "metadata_branching" => {
                    let key = step.policy_metadata_key.as_deref().unwrap_or("?");
                    format!("Metadata Branch (key: {key})")
                }
                other => other.to_string(),
            };
            fields.push((format!("Step {}", i + 1), summary));
        }
    }

    if let Some(token) = &a.incident_token {
        fields.push(("Incident Token".to_string(), token.clone()));
    }

    CommandOutput::Detail { fields }
}
