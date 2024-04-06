use futures::Future;

use crate::{
    configuration::{get_configuration, Configuration},
    db::{DbManager, IndexerDbRecording},
    executor::{Executor, ExecutorCallback},
    fetcher::{FetchingManager, TxBatch},
    indexer::{IndexerReport, IndexingResult},
    processor::ProcessingManager,
};

use {
    chrono::Utc, solana_sdk::clock::UnixTimestamp, std::time::Duration, tokio::time::sleep,
    tracing::info,
};

/// Indexer state
pub struct Indexer<E> {
    /// Responsible for fetching data from rpc node
    fetching_manager: FetchingManager<E>,

    /// Responsible for processing collected transactions
    processing_manager: ProcessingManager<E>,

    /// Responsible for database interaction
    db_manager: DbManager,

    /// Timestamp determining indexing interval
    timestamp_interval: UnixTimestamp,

    /// Indexer status report
    report: IndexerReport,

    /// Whether to perform database migration on start
    migrate: bool,
}

impl<E> Indexer<E>
where
    E: ExecutorCallback + Send + Sync + 'static,
{
    /// Creates new instance of indexer
    pub async fn build() -> IndexingResult<Self> {
        let settings = get_configuration::<Configuration>()?;
        let report = IndexerReport::default();

        // Note: three different instances are used due to problems with async runtime in some k8s containers
        let db_manager_1 = DbManager::connect(settings.db_settings.with_db())?;
        let db_manager_2 = DbManager::connect(settings.db_settings.with_db())?;
        let db_manager_3 = DbManager::connect(settings.db_settings.with_db())?;

        // Use INDEXER_MIGRATE env var first, then check for config (defaulting to false)
        let migrate: bool = match std::env::var("INDEXER_MIGRATE") {
            Ok(value) => matches!(value.as_str(), "1" | "true" | "TRUE" | "y"),
            Err(_) => settings.indexer_settings.migrate.unwrap_or_default(),
        };

        Ok(Self {
            fetching_manager: FetchingManager::new(&settings, report.clone(), db_manager_1)?,
            processing_manager: ProcessingManager::new(db_manager_2),
            timestamp_interval: settings.indexer_settings.timestamp_interval,
            db_manager: db_manager_3,
            report,
            migrate,
        })
    }

    #[cfg(test)]
    /// Creates new mock instance of indexer
    pub fn new_mock(connection_string: String, db_manager: DbManager) -> Self {
        let report = IndexerReport::default();
        let fetching_manager =
            FetchingManager::new_mock(connection_string, report.clone(), db_manager.clone());
        let processing_manager = ProcessingManager::new(db_manager.clone());

        Self {
            db_manager,
            fetching_manager,
            processing_manager,
            timestamp_interval: UnixTimestamp::default(),
            report,
            migrate: false,
        }
    }

    /// Freezes the stream for the specified indexing interval
    async fn wait(&self, timestamp: UnixTimestamp) {
        let interval = self.timestamp_interval - (Utc::now().timestamp() - timestamp);

        if interval > 0 {
            sleep(Duration::from_secs(interval as u64)).await;
        }
    }

    /// Runs batch processing
    #[tracing::instrument(
        level = "trace",
        skip(batch, self),
        fields(
            first_signature = %batch.first().expect("Invalid batch").signature,
            last_signature = %batch.last().expect("Invalid batch").signature,
        ))]
    async fn process_batch(&mut self, batch: &TxBatch) -> IndexingResult<()> {
        let txs = self.fetching_manager.fetch_batch(batch).await?;

        if !txs.is_empty() {
            self.processing_manager.process_batch(txs).await?;
        }

        Ok(())
    }

    /// Runs indexer iteration for selected signature scope
    #[tracing::instrument(
        level = "trace",
        skip(self, until),
        fields(
            timestamp = %timestamp,
            until = %until.as_ref().unwrap_or(&"latest signature".into())
        )
    )]
    async fn indexing_iteration(
        &mut self,
        until: &Option<String>,
        timestamp: UnixTimestamp,
    ) -> IndexingResult<()> {
        let mut before = None;
        loop {
            let signatures = self.fetching_manager.get_signatures(&before, until).await?;

            if signatures.is_empty() {
                break;
            }
            before = signatures.last().map(|sign| sign.signature.clone());

            self.process_batch(&signatures).await?;
        }

        Ok(())
    }

    /// Runs processing of the selected signature scope
    #[tracing::instrument(level = "debug", skip(self))]
    async fn run(&mut self) -> IndexingResult<()> {
        let mut until = None;

        // If we have configured a migration, then it's failure is migrate error
        if self.migrate {
            self.db_manager.migrate().await?;
        }

        loop {
            let iteration_timestamp = Utc::now().timestamp();
            self.indexing_iteration(&until, iteration_timestamp).await?;
            self.wait(iteration_timestamp).await;

            until = self.db_manager.get_most_recent_tx().await?;
        }
    }
}

pub trait IndexerEngine {
    type Executor;
    type Other<R>;

    fn start_indexing(&mut self) -> impl Future<Output = IndexingResult<()>> + Send;
    fn set_executor(&mut self, executor: Self::Executor);
    fn replace_excutor<R>(self, executor: R) -> Self::Other<R>
    where
        R: ExecutorCallback + Send + Sync + 'static;
    fn get_report(&self) -> IndexerReport;
}

impl<E> IndexerEngine for Indexer<E>
where
    E: ExecutorCallback + Send + Sync + 'static,
{
    type Executor = E;
    type Other<R> = Indexer<R>;

    /// Sets a callback for further processing
    fn set_executor(&mut self, executor: Self::Executor) {
        let executor = Executor::from_executor(executor);
        self.processing_manager.set_executor(executor.clone());
        self.fetching_manager.set_executor(executor);
    }

    fn replace_excutor<R>(self, executor: R) -> Indexer<R>
    where
        R: ExecutorCallback + Send + Sync + 'static,
    {
        let executor = Executor::from_executor(executor);
        let Indexer {
            fetching_manager,
            processing_manager,
            db_manager,
            timestamp_interval,
            report,
            migrate,
            ..
        } = self;

        Indexer {
            processing_manager: processing_manager.replace_executor(executor.clone()),
            fetching_manager: fetching_manager.replace_executor(executor),
            db_manager,
            timestamp_interval,
            report,
            migrate,
        }
    }

    /// Indexes all instructions that were invoked during transactions in specified program
    #[tracing::instrument(level = "debug", skip(self))]
    async fn start_indexing(&mut self) -> IndexingResult<()> {
        info!("Start indexing");
        self.report.set_available().await;

        if let Err(err) = self.run().await {
            err.get_trace();
            self.report.set_unavailable().await;

            return Err(err);
        }
        Ok(())
    }

    // Returns indexation report
    fn get_report(&self) -> IndexerReport {
        self.report.clone()
    }
}
