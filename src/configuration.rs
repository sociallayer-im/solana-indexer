use config::{Config, File};
use serde::Deserialize;

use crate::{db::DatabaseSettings, FetchingSettings, IndexerSettings, IndexingResult};

/// Configurations for the indexer engine
#[derive(Deserialize, Clone, Debug)]
pub struct Configuration {
    pub indexer_settings: IndexerSettings,
    pub db_settings: DatabaseSettings,
    pub fetcher_settings: Option<FetchingSettings>,
}

pub fn get_configuration<'de, T: Deserialize<'de>>() -> IndexingResult<T> {
    let config = std::env::var("INDEXER_CFG").unwrap_or_else(|_| "configuration.yaml".to_string());

    let builder = Config::builder()
        .add_source(File::with_name(&config).required(true))
        .build()?;

    builder.try_deserialize::<T>().map_err(Into::into)
}
