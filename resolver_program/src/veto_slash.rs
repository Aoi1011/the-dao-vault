use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{load_signer, load_system_program};
use jito_restaking_core::{ncn::Ncn, operator::Operator};
use resolver_core::{
    config::Config, ncn_slash_proposal_ticket::NcnSlashProposalTicket, resolver::Resolver,
    slash_proposal::SlashProposal, slasher::Slasher,
};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_veto_slash(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config_info, ncn_info, operator_info, slasher_info, resolver_info, slash_proposal_info, ncn_slash_proposal_ticket_info, resolver_admin_info, system_program] =
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

    Resolver::load(program_id, resolver_info, false)?;
    let resolver_data = resolver_info.data.borrow();
    let resolver = Resolver::try_from_slice_unchecked(&resolver_data)?;

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
    let ncn_slash_proposal_ticket =
        NcnSlashProposalTicket::try_from_slice_unchecked(&ncn_slash_proposal_ticket_data)?;

    load_signer(resolver_admin_info, true)?;
    load_system_program(system_program)?;

    resolver.check_admin(resolver_admin_info.key)?;

    slash_proposal.check_veto_period_ended(Clock::get()?.slot)?;
    slash_proposal.check_completed()?;

    ncn_slash_proposal_ticket.check_resolver(resolver_info.key)?;
    ncn_slash_proposal_ticket.check_slash_proposal(slash_proposal_info.key)?;

    slash_proposal.set_completed(true);

    Ok(())
}
