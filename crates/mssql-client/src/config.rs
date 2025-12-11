//! Client configuration.

use std::time::Duration;

use mssql_auth::Credentials;
use mssql_tls::TlsConfig;

/// Configuration for connecting to SQL Server.
#[derive(Debug, Clone)]
pub struct Config {
    /// Server hostname or IP address.
    pub host: String,

    /// Server port (default: 1433).
    pub port: u16,

    /// Database name.
    pub database: Option<String>,

    /// Authentication credentials.
    pub credentials: Credentials,

    /// TLS configuration.
    pub tls: TlsConfig,

    /// Application name (shown in SQL Server management tools).
    pub application_name: String,

    /// Connection timeout.
    pub connect_timeout: Duration,

    /// Command timeout.
    pub command_timeout: Duration,

    /// TDS packet size.
    pub packet_size: u16,

    /// Whether to use TDS 8.0 strict mode.
    pub strict_mode: bool,

    /// Whether to trust the server certificate.
    pub trust_server_certificate: bool,

    /// Instance name (for named instances).
    pub instance: Option<String>,

    /// Whether to enable MARS (Multiple Active Result Sets).
    pub mars: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 1433,
            database: None,
            credentials: Credentials::sql_server("", ""),
            tls: TlsConfig::default(),
            application_name: "mssql-client".to_string(),
            connect_timeout: Duration::from_secs(30),
            command_timeout: Duration::from_secs(30),
            packet_size: 4096,
            strict_mode: false,
            trust_server_certificate: false,
            instance: None,
            mars: false,
        }
    }
}

impl Config {
    /// Create a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a connection string into configuration.
    ///
    /// Supports ADO.NET-style connection strings:
    /// ```text
    /// Server=localhost;Database=mydb;User Id=sa;Password=secret;
    /// ```
    pub fn from_connection_string(conn_str: &str) -> Result<Self, crate::error::Error> {
        let mut config = Self::default();

        for part in conn_str.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let (key, value) = part
                .split_once('=')
                .ok_or_else(|| crate::error::Error::Config(format!("invalid key-value: {part}")))?;

            let key = key.trim().to_lowercase();
            let value = value.trim();

            match key.as_str() {
                "server" | "data source" | "host" => {
                    // Handle host:port or host\instance format
                    if let Some((host, port_or_instance)) = value.split_once(',') {
                        config.host = host.to_string();
                        config.port = port_or_instance.parse().map_err(|_| {
                            crate::error::Error::Config(format!("invalid port: {port_or_instance}"))
                        })?;
                    } else if let Some((host, instance)) = value.split_once('\\') {
                        config.host = host.to_string();
                        config.instance = Some(instance.to_string());
                    } else {
                        config.host = value.to_string();
                    }
                }
                "port" => {
                    config.port = value.parse().map_err(|_| {
                        crate::error::Error::Config(format!("invalid port: {value}"))
                    })?;
                }
                "database" | "initial catalog" => {
                    config.database = Some(value.to_string());
                }
                "user id" | "uid" | "user" => {
                    // Update credentials with new username
                    if let Credentials::SqlServer { password, .. } = &config.credentials {
                        config.credentials =
                            Credentials::sql_server(value.to_string(), password.clone());
                    }
                }
                "password" | "pwd" => {
                    // Update credentials with new password
                    if let Credentials::SqlServer { username, .. } = &config.credentials {
                        config.credentials =
                            Credentials::sql_server(username.clone(), value.to_string());
                    }
                }
                "application name" | "app" => {
                    config.application_name = value.to_string();
                }
                "connect timeout" | "connection timeout" => {
                    let secs: u64 = value.parse().map_err(|_| {
                        crate::error::Error::Config(format!("invalid timeout: {value}"))
                    })?;
                    config.connect_timeout = Duration::from_secs(secs);
                }
                "command timeout" => {
                    let secs: u64 = value.parse().map_err(|_| {
                        crate::error::Error::Config(format!("invalid timeout: {value}"))
                    })?;
                    config.command_timeout = Duration::from_secs(secs);
                }
                "trustservercertificate" | "trust server certificate" => {
                    config.trust_server_certificate = value.eq_ignore_ascii_case("true")
                        || value.eq_ignore_ascii_case("yes")
                        || value == "1";
                }
                "encrypt" => {
                    config.strict_mode = value.eq_ignore_ascii_case("strict");
                }
                "multipleactiveresultsets" | "mars" => {
                    config.mars = value.eq_ignore_ascii_case("true")
                        || value.eq_ignore_ascii_case("yes")
                        || value == "1";
                }
                "packet size" => {
                    config.packet_size = value.parse().map_err(|_| {
                        crate::error::Error::Config(format!("invalid packet size: {value}"))
                    })?;
                }
                _ => {
                    // Ignore unknown options for forward compatibility
                    tracing::debug!(
                        key = key,
                        value = value,
                        "ignoring unknown connection string option"
                    );
                }
            }
        }

        Ok(config)
    }

    /// Set the server host.
    #[must_use]
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Set the server port.
    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the database name.
    #[must_use]
    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    /// Set the credentials.
    #[must_use]
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = credentials;
        self
    }

    /// Set the application name.
    #[must_use]
    pub fn application_name(mut self, name: impl Into<String>) -> Self {
        self.application_name = name.into();
        self
    }

    /// Set the connect timeout.
    #[must_use]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set trust server certificate option.
    #[must_use]
    pub fn trust_server_certificate(mut self, trust: bool) -> Self {
        self.trust_server_certificate = trust;
        self.tls = self.tls.trust_server_certificate(trust);
        self
    }

    /// Enable TDS 8.0 strict mode.
    #[must_use]
    pub fn strict_mode(mut self, enabled: bool) -> Self {
        self.strict_mode = enabled;
        self.tls = self.tls.strict_mode(enabled);
        self
    }

    /// Create a new configuration with a different host (for routing).
    #[must_use]
    pub fn with_host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    /// Create a new configuration with a different port (for routing).
    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_string_parsing() {
        let config = Config::from_connection_string(
            "Server=localhost;Database=test;User Id=sa;Password=secret;",
        )
        .unwrap();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.database, Some("test".to_string()));
    }

    #[test]
    fn test_connection_string_with_port() {
        let config =
            Config::from_connection_string("Server=localhost,1434;Database=test;").unwrap();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 1434);
    }

    #[test]
    fn test_connection_string_with_instance() {
        let config =
            Config::from_connection_string("Server=localhost\\SQLEXPRESS;Database=test;").unwrap();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.instance, Some("SQLEXPRESS".to_string()));
    }
}
