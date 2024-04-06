use {
    crate::{
        fetcher::{FetchingResult, Tx},
        CallbackResult, Instruction,
    },
    futures::{lock::Mutex, Future},
    std::sync::Arc,
};

pub type TxSignature = solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
pub type TxMeta = solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

/// An entity that is called on every indexed consequentially
pub enum Executor<E = ()> {
    None,
    Executor(Arc<Mutex<E>>),
}

impl<E> Clone for Executor<E> {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Executor(arg0) => Self::Executor(arg0.clone()),
        }
    }
}

impl<E> Executor<E>
where
    E: ExecutorCallback + Send + Sync + 'static,
{
    pub fn from_executor(executor: E) -> Self {
        Self::Executor(Arc::new(Mutex::new(executor)))
    }
}

#[allow(unused_variables)]
pub trait ExecutorCallback {
    fn process_instruction(
        &mut self,
        instruction: &Instruction,
    ) -> impl Future<Output = CbResult> + Send {
        async { Ok(ExecutorControlFlow::Pass) }
    }

    fn process_parsed_transaction(&mut self, tx: &Tx) -> impl Future<Output = TxResult> + Send {
        async {
            Ok(ControlFlowWithData {
                control_flow: ExecutorControlFlow::Pass,
                data: None,
            })
        }
    }

    fn process_log_messages(
        &mut self,
        log_messages: Vec<String>,
    ) -> impl Future<Output = TxResult> + Send {
        async {
            Ok(ControlFlowWithData {
                control_flow: ExecutorControlFlow::Pass,
                data: None,
            })
        }
    }

    fn process_raw_transaction(
        &mut self,
        raw_tx: &TxMeta,
    ) -> impl Future<Output = TxResult> + Send {
        async {
            Ok(ControlFlowWithData {
                control_flow: ExecutorControlFlow::Pass,
                data: None,
            })
        }
    }

    fn process_signature(&mut self, tx: &TxSignature) -> impl Future<Output = CbResult> + Send {
        async { Ok(ExecutorControlFlow::Pass) }
    }
}

impl ExecutorCallback for () {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExecutorControlFlow {
    Skip,
    Pass,
    Stop,
}

impl Into<ExecutorControlFlow> for () {
    fn into(self) -> ExecutorControlFlow {
        ExecutorControlFlow::Pass
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ControlFlowWithData<D> {
    pub control_flow: ExecutorControlFlow,
    pub data: D,
}

pub type TxResult = CallbackResult<ControlFlowWithData<Option<FetchingResult<Tx>>>>;
pub type CbResult = CallbackResult<ExecutorControlFlow>;
