use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct PostgresSettings {
    pub host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
    application_name: String,
    pub max_connections: u32,
    tls: TLSMode,
}
impl PostgresSettings {
    pub fn new() -> Self {
        Self {
            port: 5432,
            max_connections: 50,
            application_name: "Taiko-rs Websocket Server".to_owned(),
            ..Self::default()
        }
    }
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}&application_name={}",
            self.username,
            self.password,
            self.host,
            self.port,
            self.database,
            self.tls,
            self.application_name
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TLSMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}
impl std::fmt::Display for TLSMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = match self {
            TLSMode::Disable => "disable",
            TLSMode::Allow => "allow",
            TLSMode::Prefer => "prefer",
            TLSMode::Require => "require",
            TLSMode::VerifyCa => "verify-ca",
            TLSMode::VerifyFull => "verify-full",
        };
        write!(f, "{}", m)
    }
}
impl Default for TLSMode {
    fn default() -> Self {
        Self::Prefer
    }
}