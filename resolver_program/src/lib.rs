mod execute_instant_slash;
mod execute_slash;
mod initialize_config;
mod initialize_ncn_resolver_program_config;
mod initialize_resolver;
mod propose_slash;
mod veto_slash;

use borsh::BorshDeserialize;
use resolver_sdk::instruction::ResolverInstruction;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::{
    initialize_config::process_initialize_config,
    initialize_ncn_resolver_program_config::process_initialize_resolver_program_config,
    initialize_resolver::process_initialize_resolver, propose_slash::process_propose_slash,
    veto_slash::process_veto_slash,
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

        ResolverInstruction::InitializeNcnResolverProgramConfig { veto_duration } => {
            msg!("Instruction: InitializeNcnResolverProgramConfig");
            process_initialize_resolver_program_config(program_id, accounts, veto_duration)?;
        }

        ResolverInstruction::InitializeResolver => {
            msg!("Instruction: InitializeResolver");
            process_initialize_resolver(program_id, accounts)?;
        }

        ResolverInstruction::ProposeSlash { slash_amount } => {
            msg!("Instruction: ProposeSlash");
            process_propose_slash(program_id, accounts, slash_amount)?;
        }

        ResolverInstruction::VetoSlash => {
            msg!("Instruction: VetoSlash");
            process_veto_slash(program_id, accounts)?;
        }
    }

    Ok(())
}
