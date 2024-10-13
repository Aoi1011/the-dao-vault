use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::instruction::ResolverInstruction;

pub fn initialize_config(
    program_id: &Pubkey,
    config: &Pubkey,
    admin: &Pubkey,
    restaking_program: &Pubkey,
    vault_program: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(*restaking_program, false),
        AccountMeta::new_readonly(*vault_program, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::InitializeConfig.try_to_vec().unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_resolver(
    program_id: &Pubkey,
    config: &Pubkey,
    resolver: &Pubkey,
    admin: &Pubkey,
    base: &Pubkey,
    ncn: &Pubkey,
    vault: &Pubkey,
    ncn_vault_slasher_ticket: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*resolver, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new(*base, true),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*ncn_vault_slasher_ticket, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::InitializeResolver
            .try_to_vec()
            .unwrap(),
    }
}

pub fn initialize_slash_request_list(
    program_id: &Pubkey,
    config: &Pubkey,
    slash_request_list: &Pubkey,
    admin: &Pubkey,
    ncn: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*slash_request_list, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::InitializeSlashRequestList
            .try_to_vec()
            .unwrap(),
    }
}
