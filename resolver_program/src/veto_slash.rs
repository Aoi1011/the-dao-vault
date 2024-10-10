use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_veto_slash(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}
