use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use resolver_core::slasher::Slasher;
use resolver_sdk::instruction::SlasherAdminRole;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_slasher_set_secondary_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    role: SlasherAdminRole,
) -> ProgramResult {
    let [slasher_info, admin, new_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Slasher::load(program_id, slasher_info, false)?;
    let mut slasher_data = slasher_info.data.borrow_mut();
    let slasher = Slasher::try_from_slice_unchecked_mut(&mut slasher_data)?;

    load_signer(admin, false)?;

    slasher.check_admin(admin.key)?;

    match role {
        SlasherAdminRole::DelegateAdmin => {
            slasher.delegate_admin = *new_admin.key;
        }
    }

    Ok(())
}
