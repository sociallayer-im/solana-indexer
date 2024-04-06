//! Solana transaction indexing entity
//!
//! # Examples
//!
//! ```no_run
//! use solana_indexer::{Indexer, CallbackResult, Instruction, IndexerEngine, InstructionCallback, InstructionExecutor};
//!
//! #[derive(Default)]
//!pub struct ProcessingStruct;
//!
//! impl InstructionCallback for ProcessingStruct {
//!    async fn process_instruction(&mut self, instruction: &Instruction) -> CallbackResult<()> {
//!        println!("Instruction program id: {}", instruction.program_id);
//!        Ok(())
//!    }
//!}
//!
//!#[tokio::main]
//! async fn main() {
//!     let mut solana_indexer = Indexer::build().await.unwrap();
//!
//!     let processor = ProcessingStruct::default();
//!     solana_indexer.set_executor(InstructionExecutor::from_executor(processor));
//!     solana_indexer.start_indexing().await.unwrap();
//! }
//! ```
//! # Panics
//!
//! Library panics if it fails to connect to database.
//!
//! # Environment variables
//!
//! - **INDEXER_CFG**
//!
//!   Path to configuration file.
//!   Default is `configuration.yaml`.
//!
//! - **INDEXER_MIGRATE**
//!
//!   Flag to perform database migration on startup.
//!   Acceptable true values: `1`, `true`, `y`. Everything else is false.
//!   This variable overrides `indexer_settings.migrate` setting in the configuration file.
//!   Default is false.
//!
//! # Configuration
//!
//! Indexer is configured via a configuration file specified by `INDEXER_CFG` environment variable.
//! Please refer to [Configuration] schema for details on available settings.

mod configuration;
mod db;
mod executor;
mod fetcher;
mod indexer;
mod processor;
mod utils;

pub use {
    configuration::{get_configuration, Configuration},
    executor::{
        CbResult, ControlFlowWithData, Executor, ExecutorCallback, ExecutorControlFlow, TxMeta,
        TxResult, TxSignature,
    },
    fetcher::{fetching_settings::FetchingSettings, Tx, TxBatch},
    indexer::{
        indexer_engine::{Indexer, IndexerEngine},
        indexer_error::{IndexerError, IndexingResult},
        indexer_report::{IndexerReport, IndexerState, RequestMetrics},
        indexer_settings::IndexerSettings,
    },
    processor::{
        instruction::Instruction,
        processor_error::{CallbackError, CallbackResult},
    },
    solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta,
};
