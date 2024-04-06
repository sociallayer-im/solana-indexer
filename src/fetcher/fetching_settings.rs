use serde::Deserialize;

/// Maximum amount of transaction that can be fetched from RPC node
const MAX_TRANSACTION_BATCH_SIZE: usize = 20;

/// Settings struct dedicated to fetching data from Solana RPC
#[derive(Deserialize, Clone, Debug)]
pub struct FetchingSettings {
    /// Maximum allowed duration of a RPC call in milliseconds
    pub rpc_request_timeout: u64,

    /// Maximum allowed number of retries
    pub retry_limit: u64,

    /// Amount of transaction that can be fetched in one time
    pub transaction_batch_size: usize,
}

impl Default for FetchingSettings {
    fn default() -> Self {
        FetchingSettings {
            rpc_request_timeout: 100,
            retry_limit: 10,
            transaction_batch_size: MAX_TRANSACTION_BATCH_SIZE,
        }
    }
}
