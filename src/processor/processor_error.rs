use thiserror::Error;

/// Result of the transaction processing
pub type ProcessingResult<T> = std::result::Result<T, ProcessingError>;

/// Result of the callback execution
pub type CallbackResult<T> = std::result::Result<T, CallbackError>;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum NativeProcessingError {
    #[error("Processor has empty callback")]
    EmptyCb,
    #[error("Transaction without instructions")]
    TxWithoutInstructions,
    #[error("Instruction without account keys")]
    InstructionWithoutAccounts,
}

/// An error that was caused by a library callback function
pub type CallbackError = anyhow::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error(transparent)]
    NativeProcessor(#[from] NativeProcessingError),
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
    #[error(transparent)]
    CbError(#[from] CallbackError),
}
