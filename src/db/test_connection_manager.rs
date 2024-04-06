use {
    sqlx::{
        postgres::{PgConnectOptions, PgConnection, PgRow},
        Connection, Executor, PgPool,
    },
    std::{sync::Arc, thread::sleep, time::Duration},
    tokio::sync::Mutex,
};

use crate::{
    configuration::{get_configuration, Configuration},
    db::DatabaseSettings,
};

pub struct ConnectionManager {
    pub settings: DatabaseSettings,
    pub initial_connection: Arc<Mutex<PgConnection>>,
    pub pool: PgPool,
}

impl ConnectionManager {
    pub async fn build() -> ConnectionManager {
        let mut settings = get_configuration::<Configuration>()
            .expect("Can't load configuration")
            .db_settings;

        settings.database_name = uuid::Uuid::new_v4().to_string();

        let mut initial_connection = PgConnection::connect_with(&settings.without_db())
            .await
            .expect("Failed to connect to Postgres");

        initial_connection
            .execute(
                format!(
                    r#"CREATE DATABASE "{}" WITH OWNER postgres;;"#,
                    &settings.database_name
                )
                .as_str(),
            )
            .await
            .expect("Failed to create database.");

        let pool = PgPool::connect_with(settings.with_db())
            .await
            .expect("Failed to connect to Postgres.");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to migrate the database");

        ConnectionManager {
            initial_connection: Arc::new(Mutex::new(initial_connection)),
            settings,
            pool,
        }
    }

    pub async fn execute(&mut self, query: &str) -> Vec<PgRow> {
        sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .expect("Failed to execute query")
    }

    pub fn get_connection_options(&self) -> PgConnectOptions {
        self.settings.with_db()
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        let connection = self.initial_connection.clone();
        let query = format!(
            r#"DROP DATABASE "{}" WITH (FORCE);"#,
            self.settings.database_name.clone()
        );

        let thread = tokio::spawn(async move {
            connection
                .lock()
                .await
                .execute(query.as_str())
                .await
                .expect("Failed to drop database.");
        });

        while !thread.is_finished() {
            sleep(Duration::from_millis(100));
        }
    }
}
