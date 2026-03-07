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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SqlAuthConfig {
    pub host: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DefaultsConfig {
    pub email: Option<String>,
    pub team: Option<String>,
    pub output: Option<String>,
}
