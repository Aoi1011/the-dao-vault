use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{load_signer, load_token_account, load_token_mint};
use resolver_core::slasher::Slasher;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

pub fn process_slasher_delegate_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [slasher_info, delegate_admin, token_mint, token_account, delegate, token_program_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Slasher::load(program_id, slasher_info, false)?;
    let slasher_data = slasher_info.data.borrow();
    let slasher = Slasher::try_from_slice_unchecked(&slasher_data)?;

    load_signer(delegate_admin, false)?;
    load_token_mint(token_mint)?;
    load_token_account(
        token_account,
        slasher_info.key,
        token_mint.key,
        token_program_info,
    )?;
    spl_token_2022::check_spl_token_program_account(token_program_info.key)?;

    if token_mint.owner.ne(token_account.owner) {
        return Err(ProgramError::InvalidAccountData);
    }

    slasher.check_delegate_admin(delegate_admin.key)?;

    let mut slasher_seeds = Slasher::seeds(&slasher.base);
    slasher_seeds.push(vec![slasher.bump]);
    let slasher_seeds_slice: Vec<&[u8]> =
        slasher_seeds.iter().map(|seed| seed.as_slice()).collect();

    drop(slasher_data);

    let ix = spl_token_2022::instruction::approve(
        token_program_info.key,
        token_account.key,
        delegate.key,
        slasher_info.key,
        &[],
        u64::MAX,
    )?;

    invoke_signed(
        &ix,
        &[
            token_program_info.clone(),
            token_account.clone(),
            delegate.clone(),
            slasher_info.clone(),
        ],
        &[&slasher_seeds_slice],
    )?;

    Ok(())
}
