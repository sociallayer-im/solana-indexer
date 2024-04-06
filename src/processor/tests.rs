use {
    super::*,
    crate::{
        db::{test_connection_manager::ConnectionManager, DbManager},
        fetcher::Tx,
        Executor, ExecutorCallback, ExecutorControlFlow,
    },
    enum_extract::let_extract,
    solana_transaction_status::{parse_accounts::ParsedAccount, UiCompiledInstruction},
    sqlx::postgres::PgConnectOptions,
    thiserror::Error,
};

struct TestProcessor;

impl ExecutorCallback for TestProcessor {
    async fn process_instruction(
        &mut self,
        _instruction: &Instruction,
    ) -> CallbackResult<ExecutorControlFlow> {
        Ok(ExecutorControlFlow::Pass)
    }
}

struct ErrTestProcessor;

impl ExecutorCallback for ErrTestProcessor {
    async fn process_instruction(
        &mut self,
        _instruction: &Instruction,
    ) -> CallbackResult<ExecutorControlFlow> {
        #[derive(Error, Debug)]
        pub enum CustomError {
            #[error("Custom error")]
            Custom,
        }

        Err(CustomError::Custom.into())
    }
}

async fn get_processor<E>(executor: Executor<E>, options: PgConnectOptions) -> ProcessingManager<E>
where
    E: ExecutorCallback + Send + Sync + 'static,
{
    let db_manager = DbManager::connect(options).expect("Failed to create manager");

    let mut processing_manager = ProcessingManager::<E>::new(db_manager);
    processing_manager.set_executor(executor);

    processing_manager
}

#[tokio::test(flavor = "multi_thread")]
async fn get_instructions_success_test() {
    let tx = Tx::new(
        String::default(),
        123,
        vec![UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: String::default(),
            stack_height: None,
        }],
        vec![ParsedAccount {
            pubkey: String::default(),
            writable: true,
            signer: true,
            source: None,
        }],
    );

    let connection_manager = ConnectionManager::build().await;
    let processor = Executor::from_executor(TestProcessor {});
    let processing_manager =
        get_processor(processor, connection_manager.get_connection_options()).await;

    processing_manager.get_instructions(&tx).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn no_instruction_in_tx_test() {
    let tx = Tx::new(
        String::default(),
        123,
        vec![],
        vec![ParsedAccount {
            pubkey: String::default(),
            writable: true,
            signer: true,
            source: None,
        }],
    );

    let connection_manager = ConnectionManager::build().await;
    let processor = Executor::from_executor(TestProcessor {});
    let processing_manager =
        get_processor(processor, connection_manager.get_connection_options()).await;

    let res = processing_manager.get_instructions(&tx);

    let_extract!(
        ProcessingError::NativeProcessor(err),
        res.unwrap_err(),
        panic!("Wrong error type")
    );
    assert_eq!(err, NativeProcessingError::TxWithoutInstructions)
}

#[tokio::test(flavor = "multi_thread")]
async fn no_acc_in_instruction_test() {
    let tx = Tx::new(
        String::default(),
        123,
        vec![UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: String::default(),
            stack_height: None,
        }],
        vec![],
    );

    let connection_manager = ConnectionManager::build().await;
    let processor = Executor::from_executor(TestProcessor {});
    let processing_manager =
        get_processor(processor, connection_manager.get_connection_options()).await;
    let res = processing_manager.get_instructions(&tx);

    let_extract!(
        ProcessingError::NativeProcessor(err),
        res.unwrap_err(),
        panic!("Wrong error type")
    );
    assert_eq!(err, NativeProcessingError::InstructionWithoutAccounts)
}

#[tokio::test(flavor = "multi_thread")]
async fn process_batch_success_test() {
    let txs = vec![Tx::new(
        String::default(),
        123,
        vec![UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: String::default(),
            stack_height: None,
        }],
        vec![ParsedAccount {
            pubkey: String::default(),
            writable: true,
            signer: true,
            source: None,
        }],
    )];

    let connection_manager = ConnectionManager::build().await;
    let processor = Executor::from_executor(TestProcessor {});
    let mut processing_manager =
        get_processor(processor, connection_manager.get_connection_options()).await;

    processing_manager.process_batch(txs).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[should_panic(expected = "Custom error")]
async fn process_batch_fail_test() {
    let txs = vec![Tx::new(
        String::default(),
        123,
        vec![UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0],
            data: String::default(),
            stack_height: None,
        }],
        vec![ParsedAccount {
            pubkey: String::default(),
            writable: true,
            signer: true,
            source: None,
        }],
    )];

    let connection_manager = ConnectionManager::build().await;
    let err_processor = Executor::from_executor(ErrTestProcessor {});
    let mut processing_manager =
        get_processor(err_processor, connection_manager.get_connection_options()).await;

    if let Err(err) = processing_manager.process_batch(txs).await {
        panic!("Error: {}", err);
    };
}
