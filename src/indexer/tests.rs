use {
    super::*,
    crate::{
        db::{test_connection_manager::ConnectionManager, DbManager},
        fetcher::{FetchingError, NativeFetchingError},
        processor::CallbackResult,
        ExecutorCallback, ExecutorControlFlow, Indexer, IndexerEngine, Instruction,
    },
    anyhow::anyhow,
    enum_extract::let_extract,
    http::StatusCode,
    sqlx::postgres::PgConnectOptions,
};

struct TestProcessor;

impl ExecutorCallback for TestProcessor {
    async fn process_instruction(
        &mut self,
        _instruction: &Instruction,
    ) -> CallbackResult<ExecutorControlFlow> {
        Err(anyhow!("Instruction processed"))
    }
}

struct EmptyProcessor;

impl ExecutorCallback for EmptyProcessor {
    async fn process_instruction(
        &mut self,
        _instruction: &Instruction,
    ) -> CallbackResult<ExecutorControlFlow> {
        Ok(ExecutorControlFlow::Pass)
    }
}

async fn get_indexer(url: String, options: PgConnectOptions) -> Indexer<TestProcessor> {
    let db_manager = DbManager::connect(options).expect("Failed to create manager");

    let mut indexer = Indexer::new_mock(url, db_manager);
    indexer.set_executor(TestProcessor {});
    indexer
}

#[tokio::test(flavor = "multi_thread")]
#[should_panic(expected = "Instruction processed")]
async fn start_indexing_success_test() {
    let connection_manager = ConnectionManager::build().await;
    let mut indexer = get_indexer(
        "succeeds".into(),
        connection_manager.get_connection_options(),
    )
    .await;

    indexer.start_indexing().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn start_indexing_fail_test() {
    let connection_manager = ConnectionManager::build().await;
    let mut indexer =
        get_indexer("fails".into(), connection_manager.get_connection_options()).await;

    let res = indexer.start_indexing().await;

    let_extract!(
        IndexerError::FetcherError(err),
        res.unwrap_err(),
        panic!("Wrong error type")
    );
    let_extract!(
        FetchingError::NativeFetcher(err),
        err,
        panic!("Wrong error type")
    );
    assert_eq!(err, NativeFetchingError::RpcCallLimit);
}

#[tokio::test(flavor = "multi_thread")]
async fn check_state_test() {
    let connection_manager = ConnectionManager::build().await;
    let indexer = get_indexer(
        "succeeds".into(),
        connection_manager.get_connection_options(),
    )
    .await;
    let state = indexer.get_report().get_state();

    let mut indexer = indexer.replace_excutor(EmptyProcessor {});
    let spawned_indexer = async move { indexer.start_indexing().await };

    tokio::spawn(spawned_indexer);
    assert_eq!(StatusCode::OK, *state.read().await);
}

#[tokio::test(flavor = "multi_thread")]
async fn check_fail_state_test() {
    let connection_manager = ConnectionManager::build().await;
    let mut indexer =
        get_indexer("fails".into(), connection_manager.get_connection_options()).await;

    let state = indexer.get_report().get_state();

    let res = indexer.start_indexing().await;
    assert!(res.is_err());
    assert_eq!(StatusCode::SERVICE_UNAVAILABLE, *state.read().await);
}
