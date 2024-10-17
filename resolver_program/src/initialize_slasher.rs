use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::ncn::Ncn;
use resolver_core::{config::Config, slasher::Slasher};
use resolver_sdk::error::ResolverError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

pub fn process_initialize_slasher(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [config, ncn_info, slasher_info, admin, base, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;

    Ncn::load(&config.jito_restaking_program, ncn_info, false)?;
    let ncn_data = ncn_info.data.borrow();
    let ncn = Ncn::try_from_slice_unchecked(&ncn_data)?;

    load_system_account(slasher_info, true)?;
    load_signer(admin, true)?;
    load_signer(base, false)?;
    load_system_program(system_program)?;

    let (slasher_pubkey, slasher_bump, mut slasher_seed) =
        Slasher::find_program_address(program_id, base.key);
    slasher_seed.push(vec![slasher_bump]);
    if slasher_info.key.ne(&slasher_pubkey) {
        msg!("Slasher account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Initializing slasher at address: {}", slasher_info.key);
    create_account(
        admin,
        slasher_info,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(std::mem::size_of::<Slasher>() as u64)
            .ok_or(ResolverError::ArithmeticOverflow)?,
        &slasher_seed,
    )?;

    let mut slasher_data = slasher_info.try_borrow_mut_data()?;
    slasher_data[0] = Slasher::DISCRIMINATOR;
    let slasher = Slasher::try_from_slice_unchecked_mut(&mut slasher_data)?;

    *slasher = Slasher::new(*base.key, *admin.key, ncn.slasher_count(), slasher_bump);

    Ok(())
}
