use crate::adapters::config::FileConfigStore;
use crate::adapters::http::HttpClient;

/// Holds all dependencies for command execution.
/// Built once in main, passed to every command.
pub struct AppContext {
    pub uptime: HttpClient,
    pub config: FileConfigStore,
    pub global: GlobalOptions,
}

// Fields will be used as more commands are added (e.g. team filtering, quiet mode).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GlobalOptions {
    pub output_format: OutputFormat,
    pub team: Option<String>,
    pub no_color: bool,
    pub quiet: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Csv,
}

impl std::str::FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => anyhow::bail!("Unknown output format '{}'. Expected: table, json, csv", s),
        }
    }
}
