mod execute_instant_slash;
mod execute_slash;
mod initialize_config;
mod initialize_resolver;
mod initialize_slash_request_list;
mod request_slash;
mod veto_slash;

use borsh::BorshDeserialize;
use resolver_sdk::instruction::ResolverInstruction;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::{
    initialize_config::process_initialize_config, initialize_resolver::process_initialize_resolver,
    initialize_slash_request_list::process_initialize_slash_request,
};

declare_id!("AE7fSUJSGxMzjNxSPpNTemrz9cr26RFue4GwoJ1cuR6f");

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if program_id.ne(&id()) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let instruction = ResolverInstruction::try_from_slice(instruction_data)?;

    match instruction {
        ResolverInstruction::InitializeConfig => {
            msg!("Instruction: InitializeConfig");
            process_initialize_config(program_id, accounts)?;
        }

        ResolverInstruction::InitializeResolver => {
            msg!("Instruction: InitializeResolver");
            process_initialize_resolver(program_id, accounts)?;
        }

        ResolverInstruction::InitializeSlashRequestList => {
            msg!("Instruction: InitializeSlashRequestList");
            process_initialize_slash_request(program_id, accounts)?;
        }
    }

    Ok(())
}
