use {
    solana_program::{
        bpf_loader_upgradeable, message::legacy::BUILTIN_PROGRAMS_KEYS, pubkey::Pubkey, sysvar,
    },
    solana_transaction_status::UiRawMessage,
    std::str::FromStr,
};

/// Checks if account is writable
pub fn is_acc_writable(index: usize, msg: &UiRawMessage) -> bool {
    let is_key_called_as_program = msg
        .instructions
        .iter()
        .any(|ix| ix.program_id_index == index as u8);
    let is_upgradeable_loader_present = msg.account_keys.iter().any(|key| {
        Pubkey::from_str(key).expect("Broken account address") == bpf_loader_upgradeable::id()
    });

    let demote_program_id = is_key_called_as_program && !is_upgradeable_loader_present;

    (index
        < (msg.header.num_required_signatures - msg.header.num_readonly_signed_accounts) as usize
        || (index >= msg.header.num_required_signatures as usize
            && index < msg.account_keys.len() - msg.header.num_readonly_unsigned_accounts as usize))
        && !{
            let key =
                Pubkey::from_str(msg.account_keys[index].as_str()).expect("Broken account address");
            sysvar::is_sysvar_id(&key) || BUILTIN_PROGRAMS_KEYS.contains(&key)
        }
        && !demote_program_id
}

/// Checks if account is signer
pub fn is_acc_signer(index: usize, msg: &UiRawMessage) -> bool {
    index < msg.header.num_required_signatures as usize
}

pub fn fibonacci(n: u64) -> u64 {
    if n < 2 {
        return 1;
    }

    let mut sum = 0;
    let mut last = 0;
    let mut curr = 1;
    for _i in 1..n {
        sum = last + curr;
        last = curr;
        curr = sum;
    }
    sum
}
