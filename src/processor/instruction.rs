use {
    solana_program::clock::UnixTimestamp, solana_transaction_status::parse_accounts::ParsedAccount,
};

/// Struct representing an Instruction entity from a Solana transaction
#[derive(Debug, PartialEq, Eq)]
pub struct Instruction {
    /// Sequence index in transaction
    pub id: u8,

    /// Transaction signature hash
    pub tx_hash: String,

    /// The public key of the account containing a program
    pub program_id: String,

    /// Time of transaction block
    pub blocktime: UnixTimestamp,

    /// List of encoded accounts used by the instruction
    pub account_keys: Vec<ParsedAccount>,

    /// The program input data encoded in a base-58 string
    pub data: String,
}

impl Instruction {
    pub fn new(
        id: u8,
        tx_hash: String,
        program_id: String,
        blocktime: UnixTimestamp,
        account_keys: Vec<ParsedAccount>,
        data: String,
    ) -> Instruction {
        Self {
            id,
            tx_hash,
            program_id,
            blocktime,
            account_keys,
            data,
        }
    }
}
