use {
    solana_indexer::{CallbackResult, ExecutorCallback, ExecutorControlFlow, Indexer, IndexerEngine, Instruction},
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Custom")]
    Custom,
}

#[derive(Default)]
pub struct ProcessingStruct;

impl ExecutorCallback for ProcessingStruct {
    async fn process_instruction(&mut self, instruction: &Instruction) -> CallbackResult<ExecutorControlFlow> {
        /*
        Instruction processing:
        All program input data can be pulled from instruction entity.
        Cansumer can return custom processing error: CallbackError::from_err(err)
        Or use create it from string: CallbackResult::from("CallbackResult")
        */
        if instruction.id == 123 {
            return Err(CustomError::Custom.into());
        }

        Ok(().into())
    }
}

async fn run() {
    let mut solana_indexer = Indexer::build().await.unwrap();
    let processor = ProcessingStruct::default();
    solana_indexer.set_executor(processor);

    solana_indexer.start_indexing().await.unwrap();
}

#[tokio::main]
async fn main() {
    // Note: define config file in INDEXER_CFG env variable
    tokio::try_join!(tokio::spawn(run())).unwrap();
}
