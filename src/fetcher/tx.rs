use {
    chrono::Utc,
    solana_sdk::clock::UnixTimestamp,
    solana_transaction_status::{parse_accounts::ParsedAccount, UiCompiledInstruction},
    std::fmt,
};

pub struct Tx {
    /// Transaction signature hash
    pub hash: String,

    /// Time of transaction block
    pub blocktime: UnixTimestamp,

    /// List of instructions that were invoked during transaction
    pub instructions: Vec<UiCompiledInstruction>,

    /// List of encoded accounts used by the transaction
    pub account_keys: Vec<ParsedAccount>,

    // Internal indexing status of transaction
    pub indexing_status: IndexingStatus,

    /// Timestamp when indexing was conducted
    pub indexing_timestamp: UnixTimestamp,
}

impl Tx {
    pub fn new(
        hash: String,
        blocktime: UnixTimestamp,
        instructions: Vec<UiCompiledInstruction>,
        account_keys: Vec<ParsedAccount>,
    ) -> Tx {
        Self {
            hash,
            blocktime,
            instructions,
            account_keys,
            indexing_status: IndexingStatus::Pending,
            indexing_timestamp: Utc::now().timestamp(),
        }
    }
}

impl fmt::Debug for Tx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut accounts = String::default();
        for acc in &self.account_keys {
            accounts = format!("{}, {}", accounts, acc.pubkey);
        }

        f.debug_struct("Transaction")
            .field("hash", &self.hash)
            .field("instruction_count", &self.instructions.len())
            .field("accounts", &accounts)
            .finish()
    }
}

#[derive(sqlx::Type, PartialEq, Eq, Debug)]
#[sqlx(type_name = "tx_status", rename_all = "lowercase")]
pub enum IndexingStatus {
    Pending,
    Indexed,
}
