use {
    super::*,
    crate::{
        db::test_connection_manager::ConnectionManager,
        fetcher::{IndexingStatus, Tx},
        processor::Instruction,
    },
    sqlx::Row,
};

#[tokio::test(flavor = "multi_thread")]
async fn transaction_test() {
    let mut connection_manager = ConnectionManager::build().await;
    let db_manager = DbManager::connect(connection_manager.get_connection_options())
        .expect("Failed to create manager");

    let mut tx = Tx {
        hash: "test_hash".to_string(),
        blocktime: 123,
        instructions: vec![],
        account_keys: vec![],
        indexing_status: IndexingStatus::Pending,
        indexing_timestamp: 123,
    };

    // Test insert transaction

    db_manager
        .insert_transaction(&tx)
        .await
        .expect("Failed to insert transaction");

    let rows = connection_manager
        .execute(r#"SELECT * FROM transactions WHERE hash = 'test_hash';"#)
        .await;

    assert_eq!(rows.len(), 1);

    let res = rows.get(0).unwrap();
    let hash: String = res.get("hash");
    let blocktime: i64 = res.get("blocktime");
    let indexing_status: IndexingStatus = res.get("indexing_status");
    let indexing_timestamp: i64 = res.get("indexing_timestamp");

    assert_eq!(hash, "test_hash");
    assert_eq!(blocktime, 123);
    assert_eq!(indexing_status, IndexingStatus::Pending);
    assert_eq!(indexing_timestamp, 123);

    // Test update transaction

    tx.indexing_status = IndexingStatus::Indexed;
    db_manager
        .update_transaction(&tx)
        .await
        .expect("Failed to updte transaction");

    let rows = connection_manager
        .execute(r#"SELECT * FROM transactions WHERE hash = 'test_hash';"#)
        .await;

    assert_eq!(rows.len(), 1);
    let indexing_status: IndexingStatus = rows.get(0).unwrap().get("indexing_status");
    assert_eq!(indexing_status, IndexingStatus::Indexed);
}

#[tokio::test(flavor = "multi_thread")]
async fn instruction_test() {
    let mut connection_manager = ConnectionManager::build().await;
    let db_manager = DbManager::connect(connection_manager.get_connection_options())
        .expect("Failed to create manager");

    let instruction = Instruction::new(
        1,
        "test_hash".to_string(),
        "test_id".to_string(),
        123,
        vec![],
        "empty_data".to_string(),
    );

    // Test insert instruction
    db_manager
        .insert_instruction(&instruction)
        .await
        .expect("Failed to insert instruction");

    let rows = connection_manager
        .execute(r#"SELECT * FROM instructions WHERE id = 'test_hash1';"#)
        .await;

    assert_eq!(rows.len(), 1);

    let res = rows.get(0).unwrap();
    let id: String = res.get("id");
    let tx_hash: String = res.get("tx_hash");
    let program_id: String = res.get("program_id");
    let blocktime: i64 = res.get("blocktime");
    let data: String = res.get("data");

    assert_eq!(id, "test_hash1");
    assert_eq!(tx_hash, "test_hash");
    assert_eq!(program_id, "test_id");
    assert_eq!(blocktime, 123);
    assert_eq!(data, "empty_data");
}

#[tokio::test(flavor = "multi_thread")]
async fn most_recent_tx_test() {
    let connection_manager = ConnectionManager::build().await;
    let db_manager = DbManager::connect(connection_manager.get_connection_options())
        .expect("Failed to create manager");

    let earliest_tx = Tx {
        hash: "earliest_tx".to_string(),
        blocktime: 111,
        instructions: vec![],
        account_keys: vec![],
        indexing_status: IndexingStatus::Pending,
        indexing_timestamp: 123,
    };

    let recent_tx = Tx {
        hash: "recent_tx".to_string(),
        blocktime: 123,
        instructions: vec![],
        account_keys: vec![],
        indexing_status: IndexingStatus::Pending,
        indexing_timestamp: 123,
    };

    db_manager
        .insert_transaction(&recent_tx)
        .await
        .expect("Failed to insert transaction");

    db_manager
        .insert_transaction(&earliest_tx)
        .await
        .expect("Failed to insert transaction");

    let hash = db_manager
        .get_most_recent_tx()
        .await
        .expect("Failed to get transaction")
        .expect("Transactions are absent");

    assert_eq!(hash, "recent_tx");
}

#[tokio::test(flavor = "multi_thread")]
async fn recorded_tx_test() {
    let connection_manager = ConnectionManager::build().await;
    let db_manager = DbManager::connect(connection_manager.get_connection_options())
        .expect("Failed to create manager");

    let tx = Tx {
        hash: "test_hash".to_string(),
        blocktime: 111,
        instructions: vec![],
        account_keys: vec![],
        indexing_status: IndexingStatus::Indexed,
        indexing_timestamp: 123,
    };

    db_manager
        .insert_transaction(&tx)
        .await
        .expect("Failed to insert transaction");

    assert!(db_manager
        .recorded_tx("test_hash")
        .await
        .expect("Failed to get transaction"));
}

#[tokio::test(flavor = "multi_thread")]
async fn recorded_instruction_test() {
    let connection_manager = ConnectionManager::build().await;
    let db_manager = DbManager::connect(connection_manager.get_connection_options())
        .expect("Failed to create manager");

    let instruction = Instruction::new(
        1,
        "test_hash".to_string(),
        "test_id".to_string(),
        123,
        vec![],
        "empty_data".to_string(),
    );

    // Test insert instruction
    db_manager
        .insert_instruction(&instruction)
        .await
        .expect("Failed to insert instruction");

    assert!(db_manager
        .recorded_instruction(&instruction)
        .await
        .expect("Failed to get instruction"));
}
