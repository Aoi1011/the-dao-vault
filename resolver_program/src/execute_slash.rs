use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{load_associated_token_account, load_signer, load_token_program};
use jito_restaking_core::{
    ncn::Ncn, ncn_operator_state::NcnOperatorState,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket, ncn_vault_ticket::NcnVaultTicket,
    operator::Operator, operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::{
    vault::Vault, vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket, vault_ncn_ticket::VaultNcnTicket,
    vault_operator_delegation::VaultOperatorDelegation,
};
use jito_vault_sdk::error::VaultError;
use resolver_core::{
    ncn_resolver_program_config::NcnResolverProgramConfig,
    ncn_slash_proposal_ticket::NcnSlashProposalTicket, resolver::Resolver,
    slash_proposal::SlashProposal, slasher::Slasher,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program::invoke_signed, program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_execute_slash(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config_info, ncn_resolver_program_config_info, vault_config_info, ncn_info, operator_info, slasher_info, vault_info, slasher_admin_info, ncn_operator_state_info, ncn_vault_ticket_info, operator_vault_ticket_info, vault_ncn_ticket_info, vault_operator_delegation_info, ncn_vault_slasher_ticket_info, vault_ncn_slasher_ticket_info, vault_ncn_slasher_operator_ticket_info, vault_token_account_info, slasher_token_account_info, resolver_info, slash_proposal_info, ncn_slash_proposal_ticket_info, token_program, jito_vault_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    resolver_core::config::Config::load(program_id, config_info, false)?;
    let config_data = config_info.data.borrow();
    let config = resolver_core::config::Config::try_from_slice_unchecked(&config_data)?;

    NcnResolverProgramConfig::load(
        program_id,
        ncn_resolver_program_config_info,
        ncn_info,
        false,
    )?;
    let ncn_resolver_program_config_data = ncn_resolver_program_config_info.data.borrow();
    let ncn_resolver_program_config =
        NcnResolverProgramConfig::try_from_slice_unchecked(&ncn_resolver_program_config_data)?;

    let ncn_epoch = Clock::get()?
        .slot
        .checked_div(config.epoch_length())
        .ok_or(VaultError::DivisionByZero)?;

    jito_vault_core::config::Config::load(&config.jito_vault_program, vault_config_info, false)?;
    Ncn::load(&config.jito_restaking_program, ncn_info, false)?;
    Operator::load(&config.jito_restaking_program, operator_info, false)?;

    Slasher::load(program_id, slasher_info, false)?;
    let slasher_data = slasher_info.data.borrow();
    let slasher = Slasher::try_from_slice_unchecked(&slasher_data)?;

    Vault::load(&config.jito_vault_program, vault_info, true)?;
    let vault_data = vault_info.data.borrow();
    let vault = Vault::try_from_slice_unchecked(&vault_data)?;

    load_signer(slasher_admin_info, true)?;
    NcnOperatorState::load(
        &config.jito_restaking_program,
        ncn_operator_state_info,
        ncn_info,
        operator_info,
        false,
    )?;
    NcnVaultTicket::load(
        &config.jito_restaking_program,
        ncn_vault_ticket_info,
        ncn_info,
        vault_info,
        false,
    )?;
    OperatorVaultTicket::load(
        &config.jito_restaking_program,
        operator_vault_ticket_info,
        operator_info,
        vault_info,
        false,
    )?;
    VaultNcnTicket::load(
        &config.jito_vault_program,
        vault_ncn_ticket_info,
        vault_info,
        ncn_info,
        false,
    )?;
    VaultOperatorDelegation::load(
        &config.jito_vault_program,
        vault_operator_delegation_info,
        vault_info,
        operator_info,
        true,
    )?;
    NcnVaultSlasherTicket::load(
        &config.jito_restaking_program,
        ncn_vault_slasher_ticket_info,
        ncn_info,
        vault_info,
        slasher_info,
        false,
    )?;
    VaultNcnSlasherTicket::load(
        &config.jito_vault_program,
        vault_ncn_slasher_ticket_info,
        vault_info,
        ncn_info,
        slasher_info,
        false,
    )?;
    VaultNcnSlasherOperatorTicket::load(
        &config.jito_vault_program,
        vault_ncn_slasher_operator_ticket_info,
        vault_info,
        ncn_info,
        slasher_info,
        operator_info,
        ncn_epoch,
        true,
    )?;

    load_associated_token_account(
        vault_token_account_info,
        vault_info.key,
        &vault.supported_mint,
    )?;
    load_associated_token_account(
        slasher_token_account_info,
        slasher_info.key,
        &vault.supported_mint,
    )?;

    Resolver::load(program_id, resolver_info, false)?;
    let resolver_data = resolver_info.data.borrow();
    let _resolver = Resolver::try_from_slice_unchecked(&resolver_data)?;

    SlashProposal::load(
        program_id,
        slash_proposal_info,
        ncn_info,
        operator_info,
        slasher_info,
        false,
    )?;
    let mut slash_proposal_data = slash_proposal_info.data.borrow_mut();
    let slash_proposal = SlashProposal::try_from_slice_unchecked_mut(&mut slash_proposal_data)?;

    NcnSlashProposalTicket::load(
        program_id,
        ncn_slash_proposal_ticket_info,
        ncn_info,
        slash_proposal_info,
        false,
    )?;
    let ncn_slash_proposal_ticket_data = ncn_slash_proposal_ticket_info.data.borrow();
    let _ncn_slash_proposal_ticket =
        NcnSlashProposalTicket::try_from_slice_unchecked(&ncn_slash_proposal_ticket_data)?;

    load_token_program(token_program)?;

    if jito_vault_program.key.ne(&jito_vault_program::id()) {
        msg!("jito vault program account is incorrect");
        return Err(ProgramError::InvalidAccountData);
    }

    slasher.check_admin(slasher_admin_info.key)?;

    slash_proposal.check_veto_period_not_ended(Clock::get()?.slot)?;
    slash_proposal.check_completed()?;

    slash_proposal.set_completed(true);
    slash_proposal.set_delete_deadline_slot(
        slash_proposal.delete_deadline_slot()
            + ncn_resolver_program_config.delete_slash_proposal_duration(),
    );

    let slasher_seeds = slasher.signing_seeds();
    let seed_slices: Vec<&[u8]> = slasher_seeds.iter().map(|seed| seed.as_slice()).collect();

    drop(vault_data);
    drop(slasher_data);

    let ix = jito_vault_sdk::sdk::slash(
        &config.jito_vault_program,
        vault_config_info.key,
        vault_info.key,
        ncn_info.key,
        operator_info.key,
        slasher_info.key,
        ncn_operator_state_info.key,
        ncn_vault_ticket_info.key,
        operator_vault_ticket_info.key,
        vault_ncn_ticket_info.key,
        vault_operator_delegation_info.key,
        ncn_vault_slasher_ticket_info.key,
        vault_ncn_slasher_ticket_info.key,
        vault_ncn_slasher_operator_ticket_info.key,
        vault_token_account_info.key,
        slasher_token_account_info.key,
        slash_proposal.amount(),
    );

    invoke_signed(
        &ix,
        &[
            vault_config_info.clone(),
            vault_info.clone(),
            ncn_info.clone(),
            operator_info.clone(),
            slasher_info.clone(),
            ncn_operator_state_info.clone(),
            ncn_vault_ticket_info.clone(),
            operator_vault_ticket_info.clone(),
            vault_ncn_ticket_info.clone(),
            vault_operator_delegation_info.clone(),
            ncn_vault_slasher_ticket_info.clone(),
            vault_ncn_slasher_ticket_info.clone(),
            vault_ncn_slasher_operator_ticket_info.clone(),
            vault_token_account_info.clone(),
            slasher_token_account_info.clone(),
            token_program.clone(),
        ],
        &[&seed_slices],
    )?;

    Ok(())
}
