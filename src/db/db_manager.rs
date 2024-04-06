use sqlx::{
    migrate::{MigrateError, Migrator},
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
    Error, Row,
};

use crate::{fetcher::Tx, processor::Instruction};

static MIGRATOR: Migrator = sqlx::migrate!();

type DbResult<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct DbManager {
    pool: PgPool,
}

pub trait IndexerDbRecording {
    async fn insert_transaction(&self, tx: &Tx) -> DbResult<()>;
    async fn update_transaction(&self, tx: &Tx) -> DbResult<()>;
    async fn insert_instruction(&self, instruction: &Instruction) -> DbResult<()>;
    async fn get_most_recent_tx(&self) -> DbResult<Option<String>>;
    async fn recorded_tx(&self, signature: &str) -> DbResult<bool>;
    async fn recorded_instruction(&self, instruction: &Instruction) -> DbResult<bool>;
}

impl DbManager {
    /// Creates connection to database
    pub fn connect(options: PgConnectOptions) -> DbResult<DbManager> {
        let pool = PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .max_connections(1)
            .connect_lazy_with(options);

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), MigrateError> {
        MIGRATOR.run(&self.pool).await?;
        Ok(())
    }
}

impl IndexerDbRecording for DbManager {
    /// Inserts transaction entity to db
    #[tracing::instrument(level = "debug", skip(self))]
    async fn insert_transaction(&self, tx: &Tx) -> DbResult<()> {
        sqlx::query(
            "INSERT INTO transactions (hash, blocktime, indexing_status, indexing_timestamp) \
                VALUES ($1, $2, $3, $4) ON CONFLICT (hash) DO NOTHING;",
        )
        .bind(&tx.hash)
        .bind(tx.blocktime)
        .bind(&tx.indexing_status)
        .bind(tx.indexing_timestamp)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Updates transaction indexing status in db
    #[tracing::instrument(level = "debug", skip(self))]
    async fn update_transaction(&self, tx: &Tx) -> DbResult<()> {
        sqlx::query("UPDATE transactions SET indexing_status = $1 WHERE hash = $2;")
            .bind(&tx.indexing_status)
            .bind(&tx.hash)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Inserts instruction entity to db
    #[tracing::instrument(level = "debug", skip(self))]
    async fn insert_instruction(&self, instruction: &Instruction) -> DbResult<()> {
        let id = instruction.tx_hash.clone() + instruction.id.to_string().as_str();

        sqlx::query(
            "INSERT INTO instructions (id, tx_hash, program_id, blocktime, data) \
                VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO NOTHING;",
        )
        .bind(&id)
        .bind(&instruction.tx_hash)
        .bind(&instruction.program_id)
        .bind(instruction.blocktime)
        .bind(&instruction.data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Returns transaction with most recent blockhash
    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_most_recent_tx(&self) -> DbResult<Option<String>> {
        let rows = sqlx::query("SELECT hash FROM transactions ORDER BY blocktime DESC LIMIT 1")
            .fetch_all(&self.pool)
            .await?;

        if !rows.is_empty() {
            return Ok(Some(rows[0].get("hash")));
        }

        Ok(None)
    }

    /// Checks if transaction if fully indexed
    #[tracing::instrument(level = "trace", skip(self))]
    async fn recorded_tx(&self, signature: &str) -> DbResult<bool> {
        let signature =
            sqlx::query("SELECT FROM transactions WHERE hash = $1 AND indexing_status = 'indexed'")
                .bind(signature)
                .fetch_all(&self.pool)
                .await?;

        Ok(!signature.is_empty())
    }

    /// Checks if instruction is processed
    #[tracing::instrument(level = "trace", skip(self))]
    async fn recorded_instruction(&self, instruction: &Instruction) -> DbResult<bool> {
        let id = instruction.tx_hash.clone() + instruction.id.to_string().as_str();

        let instruction = sqlx::query("SELECT FROM instructions WHERE id = $1")
            .bind(&id)
            .fetch_all(&self.pool)
            .await?;

        Ok(!instruction.is_empty())
    }
}
