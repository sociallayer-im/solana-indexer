# Solana indexer

This crate provides a transaction indexing framework for smart contracts. In order to be able to use this client, you will need to use Postgres database. Framework will be imported into an empty indexer project and contract logic for instructions will be defined by the framework consumer.

---

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
solana-indexer = "0.8"
```

It is required to set indexer configuration before start. Define config file in INDEXER_CFG env variable.
To run database migration on start define the 'migrate' field in config or set one of these values "1", "true", "TRUE", "y" in INDEXER_MIGRATE env variable.
Configuration file must contain fields:

```toml

[indexer_settings]                  # Configuration parameters for indexing engine
    program_id                      # The public key of the account containing a program
    connection_str                  # An HTTP URL of working environment
    timestamp_interval              # An interval between indexer calls
    rpc_timeout                     # Connection timeout in seconds for RPC client (optional)
    migrate                         # Boolean flag to run database migration on star (optional)

[fetching_settings]                 # Configuration of the fetching process (OPTIONAL)
    rpc_request_timeout             # Maximum allowed duration of a RPC call in milliseconds (default - 100)
    retry_limit                     # Maximum allowed number of retries (default - 10)
    transaction_batch_size          # Amount of transaction that can be fetched in one time (default - 20)

[db_settings]                       # Database configuration pameters
    host
    port
    username
    password
    database_name
    require_ssl


```

Code example:

```rust

pub use {
    solana_indexer::{
        CallbackError, CallbackResult, Indexer, IndexerEngine, IndexingResult, Instruction,
        InstructionCallback, InstructionExecutor,
    },
    std::fmt::Display,
    std::process,
    std::sync::Arc,
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Custom")]
    Custom,
}

fn unwrap_or_exit<T>(res: IndexingResult<T>) -> T {
    match res {
        Ok(value) => value,
        Err(_err) => {
            /* Error handling */
            process::exit(1);
        }
    }
}

#[derive(Default)]
pub struct ProcessingStruct;

impl InstructionCallback for ProcessingStruct {
    async fn process_instruction(&mut self, instruction: &Instruction) -> CallbackResult<()> {
        /* Callback body */
        if instruction.program_id == "" {
            // You can use your custom error as cb result
            return Err(CallbackError::from_err(CustomError::Custom));
        } else if instruction.program_id == "123" {
            // Also you can generate error from string to use it as cb result
            return Err(CallbackError::from("Callback error"));
        }
        Ok(())
    }
}

async fn run() {
    let mut solana_indexer = Indexer::build().await.unwrap();
    let executor = ProcessingStruct::default();
    solana_indexer.set_executor(InstructionExecutor::from_executor(executor));

    unwrap_or_exit(solana_indexer.start_indexing().await);
}
```

---

## Testing

Before starting `cargo test` you must run a local Postgres DB instance. Example:

```
docker run --rm -p 5432:5432 -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=indexer postgres
```

## License

Solana indexer is distributed under the terms of the MIT license.
