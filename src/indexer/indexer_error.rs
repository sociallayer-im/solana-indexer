use {
    crate::{fetcher::FetchingError, processor::ProcessingError, CallbackError},
    config::ConfigError,
    sqlx::migrate::MigrateError,
    thiserror::Error,
    tracing::error,
};

/// Result of indexing process
pub type IndexingResult<T> = std::result::Result<T, IndexerError>;

/// Collection of errors that are emitted during indexing process
#[derive(Error, Debug)]
pub enum IndexerError {
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
    #[error(transparent)]
    FetcherError(#[from] FetchingError),
    #[error(transparent)]
    ProcessorError(#[from] ProcessingError),
    #[error(transparent)]
    ConfigErr(#[from] ConfigError),
    #[error(transparent)]
    CbError(#[from] CallbackError),
}

impl IndexerError {
    pub fn get_trace(&self) {
        match self {
            IndexerError::DbError(error) => error!(error = %error, "Database query failed"),
            IndexerError::FetcherError(error) => error!(error = %error, "Batch fetching failed"),
            IndexerError::ProcessorError(error) => {
                error!(error = %error, "Batch processing failed")
            }
            IndexerError::ConfigErr(error) => {
                error!(error = %error, "Indexer configuration failed")
            }
            IndexerError::CbError(error) => error!(error = %error, "Custom error occured"),
        }
    }
}

impl From<MigrateError> for IndexerError {
    fn from(err: MigrateError) -> IndexerError {
        IndexerError::DbError(sqlx::Error::Migrate(Box::new(err)))
    }
}
