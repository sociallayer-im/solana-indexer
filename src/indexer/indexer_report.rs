use {
    http::StatusCode,
    prometheus_client::{
        encoding::EncodeLabelSet,
        metrics::{counter::Counter, family::Family},
    },
    solana_client::{
        client_error::{ClientErrorKind, Result},
        rpc_request::RpcError,
    },
    std::sync::Arc,
    tokio::sync::RwLock,
};

/// Request counting metrics
pub type RequestMetrics = Family<ResponseLabel, Counter>;

/// Indexer state for health reporting
pub type IndexerState = Arc<RwLock<StatusCode>>;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ResponseLabel {
    code: String,
}

/// A structure holding the indexer monitoring data
#[derive(Clone, Default)]
pub struct IndexerReport {
    metrics: RequestMetrics,
    state: IndexerState,
}

impl IndexerReport {
    pub fn get_state(&self) -> IndexerState {
        self.state.clone()
    }

    pub async fn set_available(&self) {
        let mut state = self.state.write().await;
        *state = StatusCode::OK;
    }

    pub async fn set_unavailable(&self) {
        let mut state = self.state.write().await;
        *state = StatusCode::SERVICE_UNAVAILABLE;
    }

    pub fn get_metrics(&self) -> RequestMetrics {
        self.metrics.clone()
    }

    pub fn inc_metrics<T>(&self, result: &Result<T>) {
        let response = if let Err(error) = result {
            if let ClientErrorKind::RpcError(RpcError::RpcResponseError { code, .. }) = &error.kind
            {
                ResponseLabel {
                    code: code.to_string(),
                }
            } else {
                ResponseLabel {
                    code: "500".to_string(),
                }
            }
        } else {
            ResponseLabel {
                code: "200".to_string(),
            }
        };

        self.metrics.get_or_create(&response).inc();
    }
}
