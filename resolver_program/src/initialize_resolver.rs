use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::{ncn::Ncn, ncn_vault_slasher_ticket::NcnVaultSlasherTicket};
use jito_vault_core::vault::Vault;
use resolver_core::{config::Config, resolver::Resolver};
use resolver_sdk::error::ResolverError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

pub fn process_initialize_resolver(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, resolver, admin, base, ncn, vault, ncn_vault_slasher_ticket, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;

    load_system_account(resolver, true)?;
    load_signer(admin, true)?;
    load_signer(base, false)?;
    Ncn::load(&config.jito_restaking_program, ncn, false)?;
    Vault::load(&config.jito_vault_program, vault, false)?;

    NcnVaultSlasherTicket::load(
        &config.jito_restaking_program,
        ncn_vault_slasher_ticket,
        ncn,
        vault,
        admin,
        false,
    )?;
    let ncn_vault_slasher_ticket_data = ncn_vault_slasher_ticket.data.borrow();
    let ncn_vault_slasher_ticket =
        NcnVaultSlasherTicket::try_from_slice_unchecked(&ncn_vault_slasher_ticket_data)?;
    load_system_program(system_program)?;

    let (resolver_pubkey, resolver_bump, mut resolver_seed) =
        Resolver::find_program_address(program_id, &base.key);
    resolver_seed.push(vec![resolver_bump]);
    if resolver.key.ne(&resolver_pubkey) {
        msg!("Resolver account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Initializing resolver at address: {}", resolver.key);
    create_account(
        admin,
        resolver,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(std::mem::size_of::<Resolver>() as u64)
            .ok_or(ResolverError::ArithmeticOverflow)?,
        &resolver_seed,
    )?;

    let mut resolver_data = resolver.try_borrow_mut_data()?;
    resolver_data[0] = Resolver::DISCRIMINATOR;
    let resolver = Resolver::try_from_slice_unchecked_mut(&mut resolver_data)?;

    *resolver = Resolver::new(
        *base.key,
        *admin.key,
        ncn_vault_slasher_ticket.index(),
        resolver_bump,
    );

    Ok(())
}
