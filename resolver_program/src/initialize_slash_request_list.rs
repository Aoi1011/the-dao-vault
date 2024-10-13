use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{load_signer, load_system_account, load_system_program},
};
use jito_restaking_core::ncn::Ncn;
use resolver_core::{config::Config, slash_request_list::SlashRequestList};
use resolver_sdk::error::ResolverError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

pub fn process_initialize_slash_request(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, slash_request_list, admin, ncn, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;

    load_system_account(slash_request_list, true)?;
    load_signer(admin, true)?;
    Ncn::load(&config.jito_restaking_program, ncn, false)?;

    load_system_program(system_program)?;

    let (slash_request_list_pubkey, slash_request_list_bump, mut slash_request_list_seed) =
        SlashRequestList::find_program_address(program_id, &ncn.key);
    slash_request_list_seed.push(vec![slash_request_list_bump]);
    if slash_request_list.key.ne(&slash_request_list_pubkey) {
        msg!("Slash request list account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    msg!(
        "Initializing resolver at address: {}",
        slash_request_list.key
    );
    create_account(
        admin,
        slash_request_list,
        system_program,
        program_id,
        &Rent::get()?,
        8_u64
            .checked_add(std::mem::size_of::<SlashRequestList>() as u64)
            .ok_or(ResolverError::ArithmeticOverflow)?,
        &slash_request_list_seed,
    )?;

    let mut slash_request_list_data = slash_request_list.try_borrow_mut_data()?;
    slash_request_list_data[0] = SlashRequestList::DISCRIMINATOR;
    let slash_request_list =
        SlashRequestList::try_from_slice_unchecked_mut(&mut slash_request_list_data)?;

    *slash_request_list = SlashRequestList::new(*ncn.key, slash_request_list_bump);

    Ok(())
}
