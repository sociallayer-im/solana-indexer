use {
    crate::CallbackError, solana_client::client_error::ClientError,
    solana_program::pubkey::ParsePubkeyError, solana_sdk::signature::ParseSignatureError,
    thiserror::Error,
};

pub type FetchingResult<T> = std::result::Result<T, FetchingError>;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum NativeFetchingError {
    #[error("Wrong type of transaction encoding")]
    WrongEncoding,
    #[error("Wrong transaction message type")]
    WrongMsgType,
    #[error("Transaction without account keys")]
    TxWithoutAccounts,
    #[error("Transaction without signatures")]
    TxWithoutSignatures,
    #[error("Transaction without blocktime")]
    TxWithoutBlocktime,
    #[error("Rpc call limit reached")]
    RpcCallLimit,
}

#[derive(Error, Debug)]
pub enum FetchingError {
    #[error(transparent)]
    NativeFetcher(#[from] NativeFetchingError),
    #[error(transparent)]
    RpcClient(#[from] ClientError),
    #[error(transparent)]
    ParseSignature(#[from] ParseSignatureError),
    #[error(transparent)]
    ParsePubkey(#[from] ParsePubkeyError),
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
    #[error(transparent)]
    CbError(#[from] CallbackError),
}
