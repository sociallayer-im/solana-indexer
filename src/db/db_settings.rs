use {
    secrecy::{ExposeSecret, Secret},
    serde::Deserialize,
    sqlx::{
        postgres::{PgConnectOptions, PgSslMode},
        ConnectOptions,
    },
    std::time::Duration,
};

#[derive(Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
    pub ssl_root_cert: Option<String>,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        let options = PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode);

        if let Some(cert) = &self.ssl_root_cert {
            options.ssl_root_cert(cert)
        } else {
            options
        }
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(tracing::log::LevelFilter::Trace)
            .log_slow_statements(tracing::log::LevelFilter::Warn, Duration::from_secs(1))
    }
}
