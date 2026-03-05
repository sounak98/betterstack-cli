use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    pub uptime_token: Option<String>,
    pub telemetry_token: Option<String>,
    #[serde(default)]
    pub sql: Option<SqlAuthConfig>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SqlAuthConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DefaultsConfig {
    pub team: Option<String>,
    pub output: Option<String>,
}
