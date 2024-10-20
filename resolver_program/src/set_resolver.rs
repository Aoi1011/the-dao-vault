use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_restaking_core::{ncn::Ncn, operator::Operator};
use resolver_core::{
    config::Config, ncn_resolver_program_config::NcnResolverProgramConfig,
    ncn_slash_proposal_ticket::NcnSlashProposalTicket, slash_proposal::SlashProposal,
    slasher::Slasher,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_set_resolver(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config_info, ncn_resolver_program_config_info, ncn_info, operator_info, slasher_info, slash_proposal_info, ncn_slash_proposal_ticket_info, ncn_resolver_admin, new_resolver_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config_info, false)?;
    let config_data = config_info.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;

    NcnResolverProgramConfig::load(
        program_id,
        ncn_resolver_program_config_info,
        ncn_info,
        false,
    )?;
    let ncn_resolver_program_config_data = ncn_resolver_program_config_info.data.borrow();
    let ncn_resolver_program_config =
        NcnResolverProgramConfig::try_from_slice_unchecked(&ncn_resolver_program_config_data)?;

    Ncn::load(&config.jito_restaking_program, ncn_info, false)?;
    Operator::load(&config.jito_restaking_program, operator_info, false)?;
    Slasher::load(program_id, slasher_info, false)?;

    SlashProposal::load(
        program_id,
        slash_proposal_info,
        ncn_info,
        operator_info,
        slasher_info,
        false,
    )?;

    NcnSlashProposalTicket::load(
        program_id,
        ncn_slash_proposal_ticket_info,
        ncn_info,
        slash_proposal_info,
        true,
    )?;
    let mut ncn_slash_proposal_ticket_data = ncn_slash_proposal_ticket_info.data.borrow_mut();
    let ncn_slash_proposal_ticket =
        NcnSlashProposalTicket::try_from_slice_unchecked_mut(&mut ncn_slash_proposal_ticket_data)?;

    load_signer(ncn_resolver_admin, true)?;

    if ncn_resolver_program_config
        .resolver_admin
        .ne(ncn_resolver_admin.key)
    {
        msg!("Admin is not the resolver admin");
        return Err(ProgramError::InvalidAccountData);
    }

    ncn_slash_proposal_ticket.set_resolver(*new_resolver_info.key);

    Ok(())
}
