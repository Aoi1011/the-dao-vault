use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{ncn::Ncn, operator::Operator};
use resolver_core::{
    config::Config, ncn_resolver_program_config::NcnResolverProgramConfig,
    ncn_slash_proposal_ticket::NcnSlashProposalTicket, slash_proposal::SlashProposal,
    slasher::Slasher,
};
use resolver_sdk::error::ResolverError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

pub fn process_propose_slash(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    slash_amount: u64,
) -> ProgramResult {
    let [config_info, ncn_resolver_program_config_info, ncn_info, operator_info, slasher_info, slash_proposal_info, ncn_slash_proposal_ticket_info, slasher_admin, system_program] =
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
    let slasher_data = slasher_info.data.borrow();
    let slasher = Slasher::try_from_slice_unchecked(&slasher_data)?;

    load_system_account(slash_proposal_info, true)?;
    load_system_account(ncn_slash_proposal_ticket_info, true)?;
    load_signer(slasher_admin, true)?;
    load_system_program(system_program)?;

    let current_slot = Clock::get()?.slot;

    slasher.check_admin(slasher_admin.key)?;

    // Initialize SlashProposal
    {
        let (slash_proposal_pubkey, slash_proposal_bump, mut slash_proposal_seed) =
            SlashProposal::find_program_address(
                program_id,
                ncn_info.key,
                operator_info.key,
                slasher_info.key,
            );
        slash_proposal_seed.push(vec![slash_proposal_bump]);
        if slash_proposal_info.key.ne(&slash_proposal_pubkey) {
            msg!("SlashProposal account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }

        msg!(
            "Initializing slash proposal at address: {}",
            slash_proposal_info.key
        );
        create_account(
            slasher_admin,
            slash_proposal_info,
            system_program,
            program_id,
            &Rent::get()?,
            8_u64
                .checked_add(std::mem::size_of::<SlashProposal>() as u64)
                .ok_or(ResolverError::ArithmeticOverflow)?,
            &slash_proposal_seed,
        )?;

        let mut slash_proposal_data = slash_proposal_info.try_borrow_mut_data()?;
        slash_proposal_data[0] = SlashProposal::DISCRIMINATOR;
        let slash_proposal = SlashProposal::try_from_slice_unchecked_mut(&mut slash_proposal_data)?;

        *slash_proposal = SlashProposal::new(
            *operator_info.key,
            *slasher_info.key,
            slash_amount,
            current_slot,
            current_slot + ncn_resolver_program_config.veto_duration(),
            slash_proposal_bump,
        );
    }

    // Initialize NcnSlashProposalTicket
    {
        let (ncn_slash_proposal_pubkey, ncn_slash_proposal_bump, mut ncn_slash_proposal_seed) =
            NcnSlashProposalTicket::find_program_address(program_id, ncn_info.key);
        ncn_slash_proposal_seed.push(vec![ncn_slash_proposal_bump]);
        if ncn_slash_proposal_ticket_info
            .key
            .ne(&ncn_slash_proposal_pubkey)
        {
            msg!("NCNSlashProposalTicket account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }

        msg!(
            "Initializing ncn slash proposal ticket at address: {}",
            ncn_slash_proposal_ticket_info.key
        );
        create_account(
            slasher_admin,
            ncn_slash_proposal_ticket_info,
            system_program,
            program_id,
            &Rent::get()?,
            8_u64
                .checked_add(std::mem::size_of::<NcnSlashProposalTicket>() as u64)
                .ok_or(ResolverError::ArithmeticOverflow)?,
            &ncn_slash_proposal_seed,
        )?;

        let mut ncn_slash_proposal_data = ncn_slash_proposal_ticket_info.try_borrow_mut_data()?;
        ncn_slash_proposal_data[0] = NcnSlashProposalTicket::DISCRIMINATOR;
        let ncn_slash_proposal =
            NcnSlashProposalTicket::try_from_slice_unchecked_mut(&mut ncn_slash_proposal_data)?;

        *ncn_slash_proposal = NcnSlashProposalTicket::new(
            *ncn_info.key,
            *slash_proposal_info.key,
            ncn_slash_proposal_bump,
        );
    }

    Ok(())
}
