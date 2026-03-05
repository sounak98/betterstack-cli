use std::io::{self, BufRead, Write};

use anyhow::Result;

use crate::adapters::config::schema::{AuthConfig, ConfigFile, DefaultsConfig};
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

fn prompt(label: &str) -> Result<String> {
    eprint!("{label}: ");
    io::stderr().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Prompt for sensitive input with terminal echo disabled.
/// Falls back to normal prompt if stdin is not a terminal (e.g. piped input).
fn prompt_secret(label: &str) -> Result<String> {
    use std::os::unix::io::AsRawFd;

    let stdin = io::stdin();
    let fd = stdin.as_raw_fd();

    // Check if stdin is a TTY
    if unsafe { libc::isatty(fd) } != 1 {
        return prompt(label);
    }

    eprint!("{label}: ");
    io::stderr().flush()?;

    // Disable echo
    let mut termios = std::mem::MaybeUninit::uninit();
    if unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) } != 0 {
        return prompt(label);
    }
    let mut termios = unsafe { termios.assume_init() };
    let original = termios;
    termios.c_lflag &= !libc::ECHO;
    unsafe { libc::tcsetattr(fd, libc::TCSANOW, &termios) };

    let mut input = String::new();
    let result = stdin.lock().read_line(&mut input);

    // Restore echo
    unsafe { libc::tcsetattr(fd, libc::TCSANOW, &original) };
    eprintln!(); // newline after hidden input

    result?;
    Ok(input.trim().to_string())
}

fn mask_token(t: &str) -> String {
    if t.len() > 8 {
        format!("{}...{}", &t[..4], &t[t.len() - 4..])
    } else {
        "set (short token)".to_string()
    }
}

async fn run_init(ctx: &AppContext) -> Result<CommandOutput> {
    eprintln!("Better Stack CLI Setup\n");
    eprintln!("Create your API tokens at:");
    eprintln!("  https://betterstack.com/settings/api-tokens/0\n");

    let uptime_token = prompt_secret("Uptime API token")?;
    if uptime_token.is_empty() {
        anyhow::bail!(
            "Uptime API token is required. Get one at https://betterstack.com/settings/api-tokens/0"
        );
    }

    let telemetry_token = prompt_secret("Telemetry API token (press Enter to skip)")?;
    let team = prompt("Default team name (press Enter to skip)")?;

    eprint!("Validating uptime token... ");
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

    let config = ConfigFile {
        auth: AuthConfig {
            uptime_token: Some(uptime_token),
            telemetry_token: if telemetry_token.is_empty() {
                None
            } else {
                Some(telemetry_token)
            },
            sql: None,
        },
        defaults: DefaultsConfig {
            team: if team.is_empty() { None } else { Some(team) },
            output: None,
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

    let team = config
        .defaults
        .team
        .unwrap_or_else(|| "not set".to_string());

    Ok(CommandOutput::Detail {
        fields: vec![
            ("Config file".to_string(), ctx.config.path_display()),
            ("Uptime token".to_string(), uptime_status),
            ("Telemetry token".to_string(), telemetry_status),
            ("Default team".to_string(), team),
        ],
    })
}
