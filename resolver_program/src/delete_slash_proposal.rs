use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::{
    close_program_account,
    loader::{load_signer, load_system_program},
};
use jito_restaking_core::{ncn::Ncn, operator::Operator};
use resolver_core::{
    config::Config, ncn_slash_proposal_ticket::NcnSlashProposalTicket,
    slash_proposal::SlashProposal, slasher::Slasher,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_delete_slash_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config_info, ncn_info, operator_info, slasher_info, slash_proposal_info, ncn_slash_proposal_ticket_info, payer, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config_info, false)?;
    let config_data = config_info.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;

    Ncn::load(&config.jito_restaking_program, ncn_info, false)?;
    Operator::load(&config.jito_restaking_program, operator_info, false)?;

    Slasher::load(program_id, slasher_info, false)?;

    SlashProposal::load(
        program_id,
        slash_proposal_info,
        ncn_info,
        operator_info,
        slasher_info,
        true,
    )?;
    let slash_proposal_data = slash_proposal_info.data.borrow();
    let slash_proposal = SlashProposal::try_from_slice_unchecked(&slash_proposal_data)?;

    NcnSlashProposalTicket::load(
        program_id,
        ncn_slash_proposal_ticket_info,
        ncn_info,
        slash_proposal_info,
        true,
    )?;

    load_signer(payer, true)?;
    load_system_program(system_program)?;

    let current_slot = Clock::get()?.slot;

    slash_proposal.check_delete_deadline_ended(current_slot)?;

    drop(slash_proposal_data);

    close_program_account(program_id, slash_proposal_info, payer)?;
    close_program_account(program_id, ncn_slash_proposal_ticket_info, payer)?;

    Ok(())
}
