use {
    super::*,
    crate::{
        db::{test_connection_manager::ConnectionManager, DbManager},
        indexer::IndexerReport,
    },
    enum_extract::let_extract,
    solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature,
    solana_sdk::message::MessageHeader,
    solana_transaction_status::{
        EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
        EncodedTransactionWithStatusMeta, UiMessage, UiParsedMessage, UiRawMessage, UiTransaction,
    },
    sqlx::postgres::PgConnectOptions,
};

async fn get_fetcher(url: String, options: PgConnectOptions) -> FetchingManager<()> {
    let db_manager = DbManager::connect(options).expect("Failed to create manager");

    FetchingManager::new_mock(url, IndexerReport::default(), db_manager)
}

async fn get_signatures(url: String) -> FetchingResult<TxBatch> {
    let connection_manager = ConnectionManager::build().await;
    let fetching_manager = get_fetcher(url, connection_manager.get_connection_options()).await;
    fetching_manager.get_signatures(&None, &None).await
}

async fn fetch_batch(url: String) -> FetchingResult<Vec<Tx>> {
    let connection_manager = ConnectionManager::build().await;
    let fetching_manager = get_fetcher(url, connection_manager.get_connection_options()).await;

    let signatures = vec![RpcConfirmedTransactionStatusWithSignature {
            signature: "3AsdoALgZFuq2oUVWrDYhg2pNeaLJKPLf8hU2mQ6U8qJxeJ6hsrPVpMn9ma39DtfYCrDQSvngWRP8NnTpEhezJpE".to_string(),
            slot: 123,
            err: None,
            memo: None,
            block_time: None,
            confirmation_status: None,
        }];

    fetching_manager.fetch_batch(&signatures).await
}

async fn create_tx(raw_tx: EncodedConfirmedTransactionWithStatusMeta) -> FetchingResult<Tx> {
    let connection_manager = ConnectionManager::build().await;
    let fetching_manager = get_fetcher(
        "succeeds".into(),
        connection_manager.get_connection_options(),
    )
    .await;
    fetching_manager.create_tx(raw_tx).await
}

#[tokio::test(flavor = "multi_thread")]
async fn get_signatures_success_test() {
    let res = get_signatures("succeeds".into()).await.unwrap();

    assert_ne!(res.len(), 0);

    for sign in res {
        assert!(!sign.signature.is_empty());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn get_signatures_fail_test() {
    let_extract!(
        FetchingError::NativeFetcher(err),
        get_signatures("fails".into()).await.unwrap_err(),
        panic!("Wrong error type")
    );
    assert_eq!(err, NativeFetchingError::RpcCallLimit);
}

#[tokio::test(flavor = "multi_thread")]
async fn fetch_batch_success_test() {
    let res = fetch_batch("succeeds".into()).await.unwrap();
    assert_eq!(res.len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn fetch_batch_fail_test() {
    let_extract!(
        FetchingError::NativeFetcher(err),
        fetch_batch("fails".into()).await.unwrap_err(),
        panic!("Wrong error type")
    );
    assert_eq!(err, NativeFetchingError::RpcCallLimit);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_tx_success_test() {
    let raw_tx = EncodedConfirmedTransactionWithStatusMeta {
            slot: 123,
            transaction: EncodedTransactionWithStatusMeta {
                transaction: EncodedTransaction::Json(UiTransaction {
                    signatures: vec!["3AsdoALgZFuq2oUVWrDYhg2pNeaLJKPLf8hU2mQ6U8qJxeJ6hsrPVpMn9ma39DtfYCrDQSvngWRP8NnTpEhezJpE".to_string()],
                    message: UiMessage::Raw(UiRawMessage {
                        header: MessageHeader {
                            num_required_signatures: 1,
                            num_readonly_signed_accounts: 1,
                            num_readonly_unsigned_accounts: 1,
                        },
                        account_keys: vec!["11111111111111111111111111111111".to_string()],
                        recent_blockhash: String::default(),
                        instructions: vec![],
                        address_table_lookups: None,
                    }),
                }),
                meta: None,
                version: None,
            },
            block_time: Some(123),
        };

    create_tx(raw_tx).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn create_wrong_encoded_tx_test() {
    let raw_tx = EncodedConfirmedTransactionWithStatusMeta {
        slot: 123,
        transaction: EncodedTransactionWithStatusMeta {
            transaction: EncodedTransaction::LegacyBinary(String::default()),
            meta: None,
            version: None,
        },
        block_time: Some(123),
    };

    let_extract!(
        FetchingError::NativeFetcher(err),
        create_tx(raw_tx).await.unwrap_err(),
        panic!("Wrong error type")
    );

    assert_eq!(err, NativeFetchingError::WrongEncoding);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_wrong_tx_msg_test() {
    let raw_tx = EncodedConfirmedTransactionWithStatusMeta {
            slot: 123,
            transaction: EncodedTransactionWithStatusMeta {
                transaction: EncodedTransaction::Json(UiTransaction {
                    signatures: vec!["3AsdoALgZFuq2oUVWrDYhg2pNeaLJKPLf8hU2mQ6U8qJxeJ6hsrPVpMn9ma39DtfYCrDQSvngWRP8NnTpEhezJpE".to_string()],
                    message: UiMessage::Parsed(UiParsedMessage {
                        account_keys: vec![],
                        recent_blockhash: String::default(),
                        instructions: vec![],
                        address_table_lookups: None,
                    }),
                }),
                meta: None,
                version: None,
            },
            block_time: Some(123),
        };

    let_extract!(
        FetchingError::NativeFetcher(err),
        create_tx(raw_tx).await.unwrap_err(),
        panic!("Wrong error type")
    );
    assert_eq!(err, NativeFetchingError::WrongMsgType);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_tx_without_accounts_test() {
    let raw_tx = EncodedConfirmedTransactionWithStatusMeta {
            slot: 123,
            transaction: EncodedTransactionWithStatusMeta {
                transaction: EncodedTransaction::Json(UiTransaction {
                    signatures: vec!["3AsdoALgZFuq2oUVWrDYhg2pNeaLJKPLf8hU2mQ6U8qJxeJ6hsrPVpMn9ma39DtfYCrDQSvngWRP8NnTpEhezJpE".to_string()],
                    message: UiMessage::Raw(UiRawMessage {
                        header: MessageHeader {
                            num_required_signatures: 1,
                            num_readonly_signed_accounts: 1,
                            num_readonly_unsigned_accounts: 1,
                        },
                        account_keys: vec![],
                        recent_blockhash: String::default(),
                        instructions: vec![],
                        address_table_lookups: None,
                    }),
                }),
                meta: None,
                version: None,
            },
            block_time: Some(123),
        };

    let_extract!(
        FetchingError::NativeFetcher(err),
        create_tx(raw_tx).await.unwrap_err(),
        panic!("Wrong error type")
    );
    assert_eq!(err, NativeFetchingError::TxWithoutAccounts);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_tx_without_signatures_test() {
    let raw_tx = EncodedConfirmedTransactionWithStatusMeta {
        slot: 123,
        transaction: EncodedTransactionWithStatusMeta {
            transaction: EncodedTransaction::Json(UiTransaction {
                signatures: vec![],
                message: UiMessage::Raw(UiRawMessage {
                    header: MessageHeader {
                        num_required_signatures: 1,
                        num_readonly_signed_accounts: 1,
                        num_readonly_unsigned_accounts: 1,
                    },
                    account_keys: vec!["11111111111111111111111111111111".to_string()],
                    recent_blockhash: String::default(),
                    instructions: vec![],
                    address_table_lookups: None,
                }),
            }),
            meta: None,
            version: None,
        },
        block_time: Some(123),
    };

    let_extract!(
        FetchingError::NativeFetcher(err),
        create_tx(raw_tx).await.unwrap_err(),
        panic!("Wrong error type")
    );
    assert_eq!(err, NativeFetchingError::TxWithoutSignatures);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_tx_without_blocktime_test() {
    let raw_tx = EncodedConfirmedTransactionWithStatusMeta {
            slot: 123,
            transaction: EncodedTransactionWithStatusMeta {
                transaction: EncodedTransaction::Json(UiTransaction {
                    signatures: vec!["3AsdoALgZFuq2oUVWrDYhg2pNeaLJKPLf8hU2mQ6U8qJxeJ6hsrPVpMn9ma39DtfYCrDQSvngWRP8NnTpEhezJpE".to_string()],
                    message: UiMessage::Raw(UiRawMessage {
                        header: MessageHeader {
                            num_required_signatures: 1,
                            num_readonly_signed_accounts: 1,
                            num_readonly_unsigned_accounts: 1,
                        },
                        account_keys: vec!["11111111111111111111111111111111".to_string()],
                        recent_blockhash: String::default(),
                        instructions: vec![],
                        address_table_lookups: None,
                    }),
                }),
                meta: None,
                version: None,
            },
            block_time: None,
        };

    let_extract!(
        FetchingError::NativeFetcher(err),
        create_tx(raw_tx).await.unwrap_err(),
        panic!("Wrong error type")
    );
    assert_eq!(err, NativeFetchingError::TxWithoutBlocktime);
}
