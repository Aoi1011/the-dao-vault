use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::instruction::{ResolverInstruction, SlasherAdminRole};

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

pub fn initialize_ncn_resolver_program_config(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    ncn_resolver_program_config: &Pubkey,
    admin: &Pubkey,
    veto_duration: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new(*ncn_resolver_program_config, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::InitializeNcnResolverProgramConfig { veto_duration }
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_resolver(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn_resolver_program_config: &Pubkey,
    ncn: &Pubkey,
    resolver: &Pubkey,
    admin: &Pubkey,
    base: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*ncn_resolver_program_config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new(*resolver, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new(*base, true),
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

pub fn initialize_slasher(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    slasher: &Pubkey,
    admin: &Pubkey,
    base: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new(*slasher, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new(*base, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::InitializeSlasher.try_to_vec().unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn propose_slash(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn_resolver_program_config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    slasher: &Pubkey,
    slash_proposal: &Pubkey,
    ncn_slash_proposal_ticket: &Pubkey,
    slasher_admin: &Pubkey,
    slash_amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn_resolver_program_config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new(*slash_proposal, false),
        AccountMeta::new(*ncn_slash_proposal_ticket, false),
        AccountMeta::new(*slasher_admin, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::ProposeSlash { slash_amount }
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn set_resolver(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    slasher: &Pubkey,
    slash_proposal: &Pubkey,
    ncn_slash_proposal_ticket: &Pubkey,
    ncn_slasher_admin: &Pubkey,
    new_resolver_info: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new_readonly(*slash_proposal, false),
        AccountMeta::new(*ncn_slash_proposal_ticket, false),
        AccountMeta::new_readonly(*ncn_slasher_admin, true),
        AccountMeta::new_readonly(*new_resolver_info, false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::SetResolver.try_to_vec().unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn veto_slash(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    slasher: &Pubkey,
    resolver: &Pubkey,
    slash_proposal: &Pubkey,
    ncn_slash_proposal_ticket: &Pubkey,
    resolver_admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new_readonly(*resolver, false),
        AccountMeta::new(*slash_proposal, false),
        AccountMeta::new(*ncn_slash_proposal_ticket, false),
        AccountMeta::new(*resolver_admin, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::VetoSlash.try_to_vec().unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn execute_slash(
    program_id: &Pubkey,
    config: &Pubkey,
    vault_config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    slasher: &Pubkey,
    vault: &Pubkey,
    slasher_admin: &Pubkey,
    ncn_operator_state: &Pubkey,
    ncn_vault_ticket: &Pubkey,
    operator_vault_ticket: &Pubkey,
    vault_ncn_ticket: &Pubkey,
    vault_operator_delegation: &Pubkey,
    ncn_vault_slasher_ticket: &Pubkey,
    vault_ncn_slasher_ticket: &Pubkey,
    vault_ncn_slasher_operator_ticket: &Pubkey,
    vault_token_account: &Pubkey,
    slasher_token_account: &Pubkey,
    resolver: &Pubkey,
    slash_proposal: &Pubkey,
    ncn_slash_proposal_ticket: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*vault_config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*slasher_admin, true),
        AccountMeta::new_readonly(*ncn_operator_state, false),
        AccountMeta::new_readonly(*ncn_vault_ticket, false),
        AccountMeta::new_readonly(*operator_vault_ticket, false), // 10
        AccountMeta::new_readonly(*vault_ncn_ticket, false),
        AccountMeta::new(*vault_operator_delegation, false),
        AccountMeta::new_readonly(*ncn_vault_slasher_ticket, false),
        AccountMeta::new_readonly(*vault_ncn_slasher_ticket, false),
        AccountMeta::new(*vault_ncn_slasher_operator_ticket, false),
        AccountMeta::new(*vault_token_account, false),
        AccountMeta::new(*slasher_token_account, false),
        AccountMeta::new_readonly(*resolver, false),
        AccountMeta::new(*slash_proposal, false),
        AccountMeta::new(*ncn_slash_proposal_ticket, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(jito_vault_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::ExecuteSlash.try_to_vec().unwrap(),
    }
}

pub fn slasher_delegate_token_account(
    program_id: &Pubkey,
    slasher: &Pubkey,
    delegate_admin: &Pubkey,
    token_mint: &Pubkey,
    token_account: &Pubkey,
    delegate: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new_readonly(*delegate_admin, true),
        AccountMeta::new(*token_mint, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new_readonly(*delegate, false),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::SlasherDelegateTokenAccount
            .try_to_vec()
            .unwrap(),
    }
}

pub fn slasher_set_admin(
    program_id: &Pubkey,
    slasher: &Pubkey,
    old_admin: &Pubkey,
    new_admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*slasher, false),
        AccountMeta::new_readonly(*old_admin, true),
        AccountMeta::new_readonly(*new_admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::SlasherSetAdmin.try_to_vec().unwrap(),
    }
}

pub fn slasher_set_secondary_admin(
    program_id: &Pubkey,
    slasher: &Pubkey,
    admin: &Pubkey,
    new_admin: &Pubkey,
    slasher_admin_role: SlasherAdminRole,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*slasher, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(*new_admin, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: ResolverInstruction::SlasherSetSecondaryAdmin(slasher_admin_role)
            .try_to_vec()
            .unwrap(),
    }
}
