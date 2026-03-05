use anyhow::Result;

use super::{prompt, prompt_secret};
use crate::adapters::config::schema::{AuthConfig, ConfigFile, DefaultsConfig, SqlAuthConfig};
use crate::context::AppContext;
use crate::output::CommandOutput;

#[derive(clap::Args)]
pub struct AuthCmd {
    #[command(subcommand)]
    command: Option<AuthSubCmd>,
}

#[derive(clap::Subcommand)]
enum AuthSubCmd {
    /// Set up authentication tokens interactively.
    Init,
    /// Show current authentication status.
    Status,
}

impl AuthCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        match &self.command {
            Some(AuthSubCmd::Init) => run_init(ctx).await,
            Some(AuthSubCmd::Status) => run_status(ctx),
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs auth", about = "Manage authentication.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<AuthSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                Ok(CommandOutput::Empty)
            }
        }
    }
}

fn mask_token(t: &str) -> String {
    if t.len() > 8 {
        format!("{}...{}", &t[..4], &t[t.len() - 4..])
    } else {
        "set (short token)".to_string()
    }
}

async fn run_init(ctx: &AppContext) -> Result<CommandOutput> {
    let existing = ctx.config.load().unwrap_or_default();
    let has_existing = ctx.config.exists();

    eprintln!("Better Stack CLI Setup\n");
    eprintln!("Create your API tokens at:");
    eprintln!("  https://betterstack.com/settings/api-tokens/0\n");

    if has_existing {
        eprintln!("Existing config found. Press Enter to keep current values.\n");
    }

    // Uptime token
    let existing_uptime = existing.auth.uptime_token.as_deref().unwrap_or("");
    let uptime_label = if existing_uptime.is_empty() {
        "Uptime API token".to_string()
    } else {
        format!("Uptime API token [{}]", mask_token(existing_uptime))
    };
    let uptime_input = prompt_secret(&uptime_label)?;
    let uptime_token = if uptime_input.is_empty() {
        if existing_uptime.is_empty() {
            anyhow::bail!(
                "Uptime API token is required. Get one at https://betterstack.com/settings/api-tokens/0"
            );
        }
        existing_uptime.to_string()
    } else {
        uptime_input
    };

    // Telemetry token
    let existing_telemetry = existing.auth.telemetry_token.as_deref().unwrap_or("");
    let telemetry_label = if existing_telemetry.is_empty() {
        "Telemetry API token (Enter to skip)".to_string()
    } else {
        format!(
            "Telemetry API token [{}] (Enter to keep)",
            mask_token(existing_telemetry)
        )
    };
    let telemetry_input = prompt_secret(&telemetry_label)?;
    let telemetry_token = if telemetry_input.is_empty() {
        existing.auth.telemetry_token.clone()
    } else {
        Some(telemetry_input)
    };

    // Team
    let existing_team = existing.defaults.team.as_deref().unwrap_or("");
    let team_label = if existing_team.is_empty() {
        "Default team name (Enter to skip)".to_string()
    } else {
        format!("Default team name [{}] (Enter to keep)", existing_team)
    };
    let team_input = prompt(&team_label)?;
    let team = if team_input.is_empty() {
        existing.defaults.team.clone()
    } else {
        Some(team_input)
    };

    // SQL Query API credentials (optional, for bs logs)
    let existing_sql = existing.auth.sql.clone().unwrap_or_default();
    let existing_sql_host = existing_sql.host.as_deref().unwrap_or("");
    let existing_sql_user = existing_sql.username.as_deref().unwrap_or("");
    let existing_sql_pass = existing_sql.password.as_deref().unwrap_or("");

    eprintln!("\nSQL Query API (for bs logs sql/query/tail):");
    eprintln!("  Get credentials from your team lead or auto-provision with a global API token.\n");

    let sql_host_label = if existing_sql_host.is_empty() {
        "SQL host (Enter to skip)".to_string()
    } else {
        format!(
            "SQL host [{}] (Enter to keep)",
            mask_token(existing_sql_host)
        )
    };
    let sql_host_input = prompt(&sql_host_label)?;

    let sql = if !sql_host_input.is_empty() || !existing_sql_host.is_empty() {
        let sql_host = if sql_host_input.is_empty() {
            existing_sql_host.to_string()
        } else {
            sql_host_input
        };

        let sql_user_label = if existing_sql_user.is_empty() {
            "SQL username".to_string()
        } else {
            format!(
                "SQL username [{}] (Enter to keep)",
                mask_token(existing_sql_user)
            )
        };
        let sql_user_input = prompt(&sql_user_label)?;
        let sql_user = if sql_user_input.is_empty() {
            existing_sql_user.to_string()
        } else {
            sql_user_input
        };

        let sql_pass_label = if existing_sql_pass.is_empty() {
            "SQL password".to_string()
        } else {
            format!(
                "SQL password [{}] (Enter to keep)",
                mask_token(existing_sql_pass)
            )
        };
        let sql_pass_input = prompt_secret(&sql_pass_label)?;
        let sql_pass = if sql_pass_input.is_empty() {
            existing_sql_pass.to_string()
        } else {
            sql_pass_input
        };

        Some(SqlAuthConfig {
            host: Some(sql_host),
            username: Some(sql_user),
            password: Some(sql_pass),
        })
    } else {
        existing.auth.sql.clone()
    };

    // Validate uptime token only if it changed
    if uptime_token != existing_uptime {
        eprint!("\nValidating uptime token... ");
        let test_client = crate::adapters::http::HttpClient::uptime(&uptime_token);
        let filters = crate::types::MonitorFilters::default();
        match test_client.list_monitors(&filters).await {
            Ok(_) => eprintln!("valid!"),
            Err(e) => {
                eprintln!("failed!");
                anyhow::bail!(
                    "Token validation failed: {}. Check your token and try again.",
                    e
                );
            }
        }
    }

    let config = ConfigFile {
        auth: AuthConfig {
            uptime_token: Some(uptime_token),
            telemetry_token,
            sql,
        },
        defaults: DefaultsConfig {
            team,
            output: existing.defaults.output,
        },
    };

    ctx.config.save(&config)?;

    Ok(CommandOutput::Message(format!(
        "\nConfiguration saved to {}",
        ctx.config.path_display()
    )))
}

fn run_status(ctx: &AppContext) -> Result<CommandOutput> {
    if !ctx.config.exists() {
        return Ok(CommandOutput::Message(
            "Not configured. Run `bs auth init` to set up.".to_string(),
        ));
    }

    let config = ctx.config.load()?;

    let uptime_status = config
        .auth
        .uptime_token
        .as_deref()
        .map(mask_token)
        .unwrap_or_else(|| "not set".to_string());

    let telemetry_status = config
        .auth
        .telemetry_token
        .as_deref()
        .map(mask_token)
        .unwrap_or_else(|| "not set".to_string());

    let sql_status = match config.auth.sql.as_ref() {
        Some(sql) if sql.host.is_some() && sql.username.is_some() => {
            let host = sql.host.as_deref().unwrap_or("");
            let user = sql.username.as_deref().unwrap_or("");
            format!("{}@{}", mask_token(user), mask_token(host))
        }
        _ => "not set".to_string(),
    };

    let team = config
        .defaults
        .team
        .unwrap_or_else(|| "not set".to_string());

    Ok(CommandOutput::Detail {
        fields: vec![
            ("Config file".to_string(), ctx.config.path_display()),
            ("Uptime token".to_string(), uptime_status),
            ("Telemetry token".to_string(), telemetry_status),
            ("SQL connection".to_string(), sql_status),
            ("Default team".to_string(), team),
        ],
    })
}
