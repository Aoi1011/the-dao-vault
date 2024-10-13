use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::ncn::Ncn;
use resolver_core::{config::Config, ncn_resolver_program_config::NcnResolverProgramConfig};
use resolver_sdk::error::ResolverError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

pub fn process_initialize_resolver_program_config(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    veto_duration: u64,
) -> ProgramResult {
    let [config, ncn, ncn_resolver_program_config, admin, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;

    Ncn::load(&config.jito_restaking_program, ncn, false)?;
    load_system_account(ncn_resolver_program_config, true)?;
    load_signer(admin, true)?;
    load_system_program(system_program)?;

    let (
        ncn_resolver_program_config_pubkey,
        ncn_resolver_program_config_bump,
        mut ncn_resolver_program_config_seeds,
    ) = NcnResolverProgramConfig::find_program_address(program_id, ncn.key);
    ncn_resolver_program_config_seeds.push(vec![ncn_resolver_program_config_bump]);
    if ncn_resolver_program_config
        .key
        .ne(&ncn_resolver_program_config_pubkey)
    {
        msg!("NcnResolverProgramConfig account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!(
        "Initializing ncn resolver program config at address {}",
        ncn_resolver_program_config.key
    );
    create_account(
        admin,
        ncn_resolver_program_config,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(std::mem::size_of::<NcnResolverProgramConfig>() as u64)
            .ok_or(ResolverError::ArithmeticOverflow)?,
        &ncn_resolver_program_config_seeds,
    )?;

    let mut ncn_resolver_program_config_data = ncn_resolver_program_config.try_borrow_mut_data()?;
    ncn_resolver_program_config_data[0] = NcnResolverProgramConfig::DISCRIMINATOR;
    let ncn_resolver_program_config = NcnResolverProgramConfig::try_from_slice_unchecked_mut(
        &mut ncn_resolver_program_config_data,
    )?;
    *ncn_resolver_program_config =
        NcnResolverProgramConfig::new(veto_duration, ncn_resolver_program_config_bump);

    Ok(())
}
