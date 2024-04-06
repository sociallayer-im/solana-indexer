use solana_transaction_status::option_serializer::OptionSerializer;

use {
    enum_extract::let_extract,
    solana_client::{
        nonblocking::rpc_client::RpcClient, rpc_client::GetConfirmedSignaturesForAddress2Config,
        rpc_response::RpcConfirmedTransactionStatusWithSignature,
    },
    solana_program::pubkey::Pubkey,
    solana_sdk::{commitment_config::CommitmentConfig, signature::Signature},
    solana_transaction_status::{
        parse_accounts::ParsedAccount, EncodedConfirmedTransactionWithStatusMeta,
        EncodedTransaction, UiMessage, UiTransactionEncoding,
    },
    std::{str::FromStr, time::Duration},
    tokio::time::sleep,
    tracing::info,
};

use crate::{
    configuration::Configuration,
    db::{DbManager, IndexerDbRecording},
    fetcher::{FetchingResult, FetchingSettings, NativeFetchingError, Tx},
    indexer::IndexerReport,
    utils::{fibonacci, is_acc_signer, is_acc_writable},
    Executor, ExecutorCallback,
};

pub type TxBatch = Vec<RpcConfirmedTransactionStatusWithSignature>;

pub struct FetchingManager<E> {
    /// A client of a remote Solana node
    rpc_client: RpcClient,

    /// The public key of the account containing a program
    program_id: Pubkey,

    /// Settings for fetching
    fetching_settings: FetchingSettings,

    /// Indexer status report
    report: IndexerReport,

    /// Responsible for database interaction
    db_manager: DbManager,

    /// Executor
    executor: Executor<E>,
}

impl<E> FetchingManager<E>
where
    E: ExecutorCallback + Send + Sync + 'static,
{
    /// This method initialize new instance of fetching manager
    pub fn new(
        config: &Configuration,
        report: IndexerReport,
        db_manager: DbManager,
    ) -> FetchingResult<Self> {
        let fetching_settings = if let Some(settings) = &config.fetcher_settings {
            settings.clone()
        } else {
            FetchingSettings::default()
        };

        let rpc_timeout = match config.indexer_settings.rpc_timeout {
            Some(seconds) => Duration::from_secs(seconds),
            None => Duration::from_secs(10),
        };

        Ok(Self {
            rpc_client: RpcClient::new_with_timeout_and_commitment(
                config.indexer_settings.connection_str.clone(),
                rpc_timeout,
                CommitmentConfig::confirmed(),
            ),
            program_id: Pubkey::from_str(&config.indexer_settings.program_id)?,
            fetching_settings,
            report,
            db_manager,
            executor: Executor::None,
        })
    }

    /// Creates new mock instance of indexer
    #[cfg(test)]
    pub fn new_mock(connection_str: String, report: IndexerReport, db_manager: DbManager) -> Self {
        Self {
            rpc_client: RpcClient::new_mock(connection_str),
            program_id: Pubkey::default(),
            fetching_settings: FetchingSettings::default(),
            report,
            db_manager,
            executor: Executor::None,
        }
    }

    pub fn set_executor(&mut self, executor: Executor<E>) {
        self.executor = executor;
    }

    pub fn replace_executor<R>(self, executor: Executor<R>) -> FetchingManager<R>
    where
        R: ExecutorCallback + Send + Sync + 'static,
    {
        let FetchingManager {
            rpc_client,
            program_id,
            fetching_settings,
            report,
            db_manager,
            ..
        } = self;
        FetchingManager {
            executor,
            rpc_client,
            program_id,
            fetching_settings,
            report,
            db_manager,
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn retry_delay(&self, error_occured: u64) -> FetchingResult<()> {
        if error_occured >= self.fetching_settings.retry_limit {
            return Err(NativeFetchingError::RpcCallLimit.into());
        }

        let delay = self.fetching_settings.rpc_request_timeout * fibonacci(error_occured);
        info!("Rpc call retry delay for {} milliseconds", delay);
        sleep(Duration::from_millis(delay)).await;

        Ok(())
    }

    /// Returns scope of signatures predetermined by the batch size
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_signatures(
        &self,
        before: &Option<String>,
        until: &Option<String>,
    ) -> FetchingResult<TxBatch> {
        let mut error_occured = 0;
        let (mut sign_before, mut sign_until) = (None, None);

        if let Some(sign) = before {
            sign_before = Some(Signature::from_str(sign.as_str())?);
        };
        if let Some(sign) = until {
            sign_until = Some(Signature::from_str(sign.as_str())?);
        };

        loop {
            let result = self.get_signatures_page(sign_before, sign_until).await;

            if let Some(signatures) = result {
                return Ok(signatures);
            } else {
                error_occured += 1;
                self.retry_delay(error_occured).await?;
            }
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_signatures_page(
        &self,
        sign_before: Option<Signature>,
        sign_until: Option<Signature>,
    ) -> Option<Vec<RpcConfirmedTransactionStatusWithSignature>> {
        let config = GetConfirmedSignaturesForAddress2Config {
            before: sign_before,
            until: sign_until,
            limit: Some(self.fetching_settings.transaction_batch_size),
            commitment: Some(CommitmentConfig::confirmed()),
        };

        let result = self
            .rpc_client
            .get_signatures_for_address_with_config(&self.program_id, config)
            .await;

        self.report.inc_metrics(&result);

        match result {
            Ok(signatures) => {
                self.report.set_available().await;
                tracing::debug!(count = %signatures.len(), "Fetched signatures");
                Some(signatures)
            }
            Err(error) => {
                self.report.set_unavailable().await;
                tracing::debug!(error = %error, "Couldn't fetch signatures");
                None
            }
        }
    }

    /// Returns list of fetched transactions
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn fetch_batch(&self, confirmed_signatures: &TxBatch) -> FetchingResult<Vec<Tx>> {
        let mut txs = vec![];

        for sign in confirmed_signatures {
            if let Executor::Executor(ref e) = self.executor {
                let mut executor = e.lock().await;
                let control_flow = executor.process_signature(sign).await?;
                match control_flow {
                    crate::ExecutorControlFlow::Skip => continue,
                    crate::ExecutorControlFlow::Pass => (),
                    crate::ExecutorControlFlow::Stop => break,
                };
            }

            if self.db_manager.recorded_tx(&sign.signature).await? {
                continue;
            }
            let signature = Signature::from_str(sign.signature.as_str())?;
            txs.push(self.fetch_tx(signature).await?)
        }
        Ok(txs)
    }

    /// Returns fetched transaction
    #[tracing::instrument(level = "trace", skip(self))]
    async fn fetch_tx(&self, signature: Signature) -> FetchingResult<Tx> {
        let mut error_occured = 0;

        loop {
            let result = self
                .rpc_client
                .get_transaction(&signature, UiTransactionEncoding::Json)
                .await;

            self.report.inc_metrics(&result);

            match result {
                Ok(mut raw_tx) => {
                    self.report.set_available().await;
                    tracing::debug!("Fetched transaction");

                    if let Executor::Executor(ref e) = self.executor {
                        let mut executor = e.lock().await;
                        let res = executor.process_raw_transaction(&raw_tx).await?;
                        match res.control_flow {
                            crate::ExecutorControlFlow::Skip => continue,
                            crate::ExecutorControlFlow::Pass => (),
                            crate::ExecutorControlFlow::Stop => match res.data {
                                Some(res) => break res,
                                None => {
                                    break Err(anyhow::anyhow!(
                                        "fetch_tx failed in process_raw_transaction executor."
                                    )
                                    .into())
                                }
                            },
                        };

                        if let Some(messges) =
                            raw_tx.transaction.meta.take().map(|mate| mate.log_messages)
                        {
                            if let OptionSerializer::Some(msgs) = messges {
                                let res = executor.process_log_messages(msgs).await?;
                                match res.control_flow {
                                    crate::ExecutorControlFlow::Skip => continue,
                                    crate::ExecutorControlFlow::Pass => (),
                                    crate::ExecutorControlFlow::Stop => match res.data {
                                        Some(res) => break res,
                                        None => {
                                            break Err(anyhow::anyhow!(
                                                "fetch_tx failed in process_log_messages executor."
                                            )
                                            .into())
                                        }
                                    },
                                };
                            }
                        }
                    }

                    let tx = self.create_tx(raw_tx).await?;

                    if let Executor::Executor(ref e) = self.executor {
                        let mut executor = e.lock().await;
                        let res = executor.process_parsed_transaction(&tx).await?;
                        match res.control_flow {
                            crate::ExecutorControlFlow::Skip => continue,
                            crate::ExecutorControlFlow::Pass => (),
                            crate::ExecutorControlFlow::Stop => match res.data {
                                Some(res) => break res,
                                None => {
                                    break Err(anyhow::anyhow!(
                                        "fetch_tx failed in process_parsed_transaction executor."
                                    )
                                    .into())
                                }
                            },
                        };
                    }

                    self.db_manager.insert_transaction(&tx).await?;
                    return Ok(tx);
                }
                Err(error) => {
                    self.report.set_unavailable().await;
                    tracing::debug!(error = %error, "Couldn't fetch transaction");

                    error_occured += 1;
                    // Note: rpc has its own timeout - 30 sec
                    self.retry_delay(error_occured).await?;
                }
            }
        }
    }

    /// Creates single transaction
    #[tracing::instrument(level = "trace", skip(self, confirmed_tx))]
    pub(crate) async fn create_tx(
        &self,
        confirmed_tx: EncodedConfirmedTransactionWithStatusMeta,
    ) -> FetchingResult<Tx> {
        let_extract!(
            EncodedTransaction::Json(tx),
            confirmed_tx.transaction.transaction,
            return Err(NativeFetchingError::WrongEncoding.into())
        );
        let_extract!(
            UiMessage::Raw(msg),
            tx.message,
            return Err(NativeFetchingError::WrongMsgType.into())
        );

        if msg.account_keys.is_empty() {
            return Err(NativeFetchingError::TxWithoutAccounts.into());
        }

        let account_keys = msg
            .account_keys
            .iter()
            .enumerate()
            .map(|(index, pubkey)| ParsedAccount {
                pubkey: pubkey.clone(),
                writable: is_acc_writable(index, &msg),
                signer: is_acc_signer(index, &msg),
                source: None,
            })
            .collect();

        let_extract!(
            Some(hash),
            tx.signatures.first(),
            return Err(NativeFetchingError::TxWithoutSignatures.into())
        );
        let_extract!(
            Some(blocktime),
            confirmed_tx.block_time,
            return Err(NativeFetchingError::TxWithoutBlocktime.into())
        );

        let tx = Tx::new(hash.clone(), blocktime, msg.instructions, account_keys);

        Ok(tx)
    }
}
