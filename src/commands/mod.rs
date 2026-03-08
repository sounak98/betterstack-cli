pub mod auth;
pub mod heartbeat_groups;
pub mod heartbeats;
pub mod incidents;
pub mod logs;
pub mod monitor_groups;
pub mod monitors;
pub mod oncall;
pub mod policies;
pub mod severities;
pub mod sources;
pub mod status;
pub mod status_pages;
pub mod upgrade;

pub use auth::AuthCmd;
pub use heartbeat_groups::HeartbeatGroupsCmd;
pub use heartbeats::HeartbeatsCmd;
pub use incidents::IncidentsCmd;
pub use logs::LogsCmd;
pub use monitor_groups::MonitorGroupsCmd;
pub use monitors::MonitorsCmd;
pub use oncall::OnCallCmd;
pub use policies::PoliciesCmd;
pub use severities::SeveritiesCmd;
pub use sources::SourcesCmd;
pub use status::StatusCmd;
pub use status_pages::StatusPagesCmd;

use std::io::{self, BufRead, Write};

pub(crate) fn prompt(label: &str) -> anyhow::Result<String> {
    eprint!("{label}: ");
    io::stderr().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub(crate) fn prompt_secret(label: &str) -> anyhow::Result<String> {
    use std::os::unix::io::AsRawFd;

    let stdin = io::stdin();
    let fd = stdin.as_raw_fd();

    if unsafe { libc::isatty(fd) } != 1 {
        return prompt(label);
    }

    eprint!("{label}: ");
    io::stderr().flush()?;

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

    unsafe { libc::tcsetattr(fd, libc::TCSANOW, &original) };
    eprintln!();

    result?;
    Ok(input.trim().to_string())
}
