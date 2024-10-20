use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use resolver_core::slasher::Slasher;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_slasher_set_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [slasher_info, old_admin, new_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Slasher::load(program_id, slasher_info, false)?;
    let mut slasher_data = slasher_info.data.borrow_mut();
    let slasher = Slasher::try_from_slice_unchecked_mut(&mut slasher_data)?;

    load_signer(old_admin, false)?;
    load_signer(new_admin, false)?;

    slasher.check_admin(old_admin.key)?;

    slasher.admin = *new_admin.key;

    slasher.update_secondary_admin(old_admin.key, new_admin.key);

    Ok(())
}
