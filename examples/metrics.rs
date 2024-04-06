use {
    prometheus_client::{encoding::text::encode, registry::Registry},
    solana_indexer::{
        CallbackResult, ExecutorCallback, ExecutorControlFlow, Indexer, IndexerEngine,
        IndexerReport, Instruction,
    },
    std::time::Duration,
    tokio::time::sleep,
};

#[derive(Default)]
pub struct ProcessingStruct;

impl ExecutorCallback for ProcessingStruct {
    async fn process_instruction(
        &mut self,
        _instruction: &Instruction,
    ) -> CallbackResult<ExecutorControlFlow> {
        Ok(().into())
    }
}

async fn show_metrics(report: IndexerReport) {
    // Can be used to track indexer status code
    let state = report.get_state();

    // Can be used to track response status of RPC node during indexing process
    let metrics = report.get_metrics();

    let mut registry = Registry::default();
    registry.register("requests", "Count of requests", metrics);

    loop {
        println!("Indexing status:\n{:?}", state.read().await);

        let mut encoded = String::new();
        encode(&mut encoded, &registry).unwrap();
        println!("Metrics output:\n{:?}", encoded);

        sleep(Duration::from_secs(10)).await;
    }
}

async fn indexing(mut solana_indexer: Indexer<ProcessingStruct>) {
    let processor = ProcessingStruct::default();
    solana_indexer.set_executor(processor);
    solana_indexer.start_indexing().await.unwrap();
}

#[tokio::main]
async fn main() {
    // Note: define config file in INDEXER_CFG env variable
    let solana_indexer = Indexer::build().await.unwrap();
    let report = solana_indexer.get_report();

    tokio::join!(indexing(solana_indexer), show_metrics(report));
}
