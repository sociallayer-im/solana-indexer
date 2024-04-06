use {serde::Deserialize, solana_sdk::clock::UnixTimestamp};

/// A structure for a general indexer configuration
#[derive(Deserialize, Clone, Debug)]
pub struct IndexerSettings {
    /// The public key of the account containing a program
    pub program_id: String,

    /// An HTTP URL of working environment
    pub connection_str: String,

    /// An interval between indexer calls
    pub timestamp_interval: UnixTimestamp,

    /// Connection timeout in seconds
    pub rpc_timeout: Option<u64>,

    /// Whether to run database migration on start
    pub migrate: Option<bool>,
}
