use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, rent::Rent,
    system_instruction, sysvar::Sysvar,
};

pub fn process_execute_slash(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [mint_account, mint_authority, metadata, payer, token_program, mpl_token_metadata_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Ok(())
}
