use {
    super::{Instruction, NativeProcessingError, ProcessingResult},
    crate::{
        db::{DbManager, IndexerDbRecording},
        fetcher::{IndexingStatus, Tx},
        Executor, ExecutorCallback,
    },
    solana_program::pubkey,
    tracing::{debug, info},
};

/// A manager that handles calling the callback function for fetched instructions and storing indexing state
pub struct ProcessingManager<E> {
    /// Entity for instruction processing
    executor: Executor<E>,

    /// Responsible for database interaction
    db_manager: DbManager,
}

impl<E> ProcessingManager<E>
where
    E: ExecutorCallback + Send + Sync + 'static,
{
    pub fn new(db_manager: DbManager) -> Self {
        ProcessingManager {
            executor: Executor::None,
            db_manager,
        }
    }

    /// Sets a Processor entity for further processing
    pub fn set_executor(&mut self, executor: Executor<E>) {
        self.executor = executor;
    }

    pub fn replace_executor<R>(self, executor: Executor<R>) -> ProcessingManager<R>
    where
        R: ExecutorCallback + Send + Sync + 'static,
    {
        ProcessingManager {
            executor,
            db_manager: self.db_manager,
        }
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn process_tx(&mut self, tx: &Tx) -> ProcessingResult<()> {
        for instruction in self.get_instructions(tx)? {
            debug!(
                tx_hash = instruction.tx_hash,
                id = instruction.id,
                "Processing instruction",
            );
            if !self.db_manager.recorded_instruction(&instruction).await? {
                if let Executor::Executor(executor) = &self.executor {
                    let control_flow = executor
                        .lock()
                        .await
                        .process_instruction(&instruction)
                        .await?;
                    match control_flow {
                        crate::ExecutorControlFlow::Skip => continue,
                        crate::ExecutorControlFlow::Pass => (),
                        crate::ExecutorControlFlow::Stop => break,
                    }
                } else {
                    return Err(NativeProcessingError::EmptyCb.into());
                }

                self.db_manager.insert_instruction(&instruction).await?;
                debug!("Instruction processed");
            }
        }
        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        skip(self, txs),
        fields(
            first_tx = %txs.first().expect("Invalid batch").hash,
            last_tx = %txs.last().expect("Invalid batch").hash,
            count = %txs.len()
        )
    )]
    /// Imposes a callback on the instructions of transaction in job
    pub async fn process_batch(&mut self, txs: Vec<Tx>) -> ProcessingResult<()> {
        for mut tx in txs {
            self.process_tx(&tx).await?;
            tx.indexing_status = IndexingStatus::Indexed;
            self.db_manager.update_transaction(&tx).await?;
            info!(tx_hash = tx.hash, "Transaction indexed");
        }
        Ok(())
    }

    #[tracing::instrument(
        level = "debug",
        skip(self),
        fields(
            tx_hash = %tx.hash,
        )
    )]
    /// Extends sequence of instructions from a single transaction
    pub(crate) fn get_instructions(&self, tx: &Tx) -> ProcessingResult<Vec<Instruction>> {
        let mut instructions = vec![];

        if tx.instructions.is_empty() {
            return Err(NativeProcessingError::TxWithoutInstructions.into());
        }

        for (id, instruction) in tx.instructions.iter().enumerate() {
            // if instruction.accounts.is_empty() {
            //     debug!("{instruction:#?}");
            //     return Err(NativeProcessingError::InstructionWithoutAccounts.into());
            // }

            let account_keys = instruction
                .accounts
                .iter()
                .map(|&index| tx.account_keys[index as usize].clone())
                .collect::<Vec<_>>();

            let program_id = pubkey!(tx.account_keys[instruction.program_id_index as usize]
                .pubkey
                .clone());

            instructions.push(Instruction::new(
                id as u8,
                tx.hash.clone(),
                program_id,
                tx.blocktime,
                account_keys,
                instruction.data.clone(),
            ));
        }
        Ok(instructions)
    }
}
