use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::ncn::Ncn;
use resolver_core::{
    config::Config, ncn_resolver_program_config::NcnResolverProgramConfig, resolver::Resolver,
};
use resolver_sdk::error::ResolverError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

pub fn process_initialize_resolver(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, ncn_resolver_program_config, ncn, resolver_info, admin, base, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;

    NcnResolverProgramConfig::load(program_id, ncn_resolver_program_config, ncn, true)?;
    let mut ncn_resolver_program_config_data = ncn_resolver_program_config.data.borrow_mut();
    let ncn_resolver_program_config = NcnResolverProgramConfig::try_from_slice_unchecked_mut(
        &mut ncn_resolver_program_config_data,
    )?;

    Ncn::load(&config.jito_restaking_program, ncn, false)?;
    load_system_account(resolver_info, true)?;
    load_signer(admin, true)?;
    load_signer(base, false)?;
    load_system_program(system_program)?;

    let (resolver_pubkey, resolver_bump, mut resolver_seed) =
        Resolver::find_program_address(program_id, base.key);
    resolver_seed.push(vec![resolver_bump]);
    if resolver_info.key.ne(&resolver_pubkey) {
        msg!("Resolver account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Initializing resolver at address: {}", resolver_info.key);
    create_account(
        admin,
        resolver_info,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(std::mem::size_of::<Resolver>() as u64)
            .ok_or(ResolverError::ArithmeticOverflow)?,
        &resolver_seed,
    )?;

    let mut resolver_data = resolver_info.try_borrow_mut_data()?;
    resolver_data[0] = Resolver::DISCRIMINATOR;
    let resolver = Resolver::try_from_slice_unchecked_mut(&mut resolver_data)?;

    *resolver = Resolver::new(
        *base.key,
        *admin.key,
        ncn_resolver_program_config.resolver_count(),
        resolver_bump,
    );

    ncn_resolver_program_config.increment_resolver_count();

    Ok(())
}
