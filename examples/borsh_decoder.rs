use {
    anyhow::anyhow,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_indexer::{CallbackResult, ExecutorCallback, ExecutorControlFlow, Indexer, IndexerEngine, Instruction},
};

/// Byte index of bump in account data - it depends on serializer
const OFFSET_BUMP: usize = 8;

/// In this example account was serialized with Borsh
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DecodedInstructionData {/* Program input data */}

#[derive(Default)]
pub struct ProcessingStruct;

impl ExecutorCallback for ProcessingStruct {
    async fn process_instruction(&mut self, instruction: &Instruction) -> CallbackResult<ExecutorControlFlow> {
        if instruction.program_id == "your program id" {
            // Note: This example checks all possible program instructions
            // To get instruction name - decode first 8 bytes of instruction data

            let buf = &bs58::decode(&instruction.data).into_vec().unwrap();
            let res = DecodedInstructionData::try_from_slice(&buf[OFFSET_BUMP..]);

            if res.is_err() {
                return Err(anyhow!("Deserialization error"));
            }
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
