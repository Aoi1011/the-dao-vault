use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{types::PodU64, AccountDeserialize, Discriminator};
use resolver_sdk::error::ResolverError;
use shank::{ShankAccount, ShankType};
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

#[derive(Debug, Clone, Copy, Zeroable, ShankType, Pod, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct Slasher {
    /// The base pubkey used as a seed for the PDA
    pub base: Pubkey,

    /// The admin pubkey
    pub admin: Pubkey,

    /// The delegate admin can delegate assets from the slasher
    pub delegate_admin: Pubkey,

    /// The slasher index
    index: PodU64,

    /// The bump seed for the PDA
    pub bump: u8,
}

impl Discriminator for Slasher {
    const DISCRIMINATOR: u8 = 4;
}

impl Slasher {
    pub fn new(base: Pubkey, admin: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            base,
            admin,
            delegate_admin: admin,
            index: PodU64::from(index),
            bump,
        }
    }

    pub fn index(&self) -> u64 {
        self.index.into()
    }

    pub fn check_admin(&self, candidate_slasher_admin: &Pubkey) -> Result<(), ResolverError> {
        if self.admin.ne(candidate_slasher_admin) {
            msg!("Slasher admin is incorrect");
            return Err(ResolverError::SlasherAdminInvalid);
        }

        Ok(())
    }

    /// Validates the delegate_admin account and ensures it matches the expected delegate_admin.
    ///
    /// # Arguments
    /// * `delegate_admin` - A reference to the [`Pubkey`] representing the delegate_admin Pubkey that is attempting
    ///   to authorize the operation.
    ///
    /// # Returns
    /// * `Result<(), RestakingError>` - Returns `Ok(())` if the delegate_admin account is valid.
    ///
    /// # Errors
    /// This function will return a [`resolver_sdk::error::ResolverError::SlasherDelegateAdminInvalid`] error in the following case:
    /// * The `delegate_admin_info` account's public key does not match the expected admin public key stored in `self`.
    pub fn check_delegate_admin(&self, delegate_admin: &Pubkey) -> Result<(), ResolverError> {
        if self.delegate_admin.ne(delegate_admin) {
            msg!(
                "Incorrect delegate_admin provided, expected {}, received {}",
                self.delegate_admin,
                delegate_admin
            );
            return Err(ResolverError::SlasherDelegateAdminInvalid);
        }
        Ok(())
    }

    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"slasher".to_vec(), base.as_ref().to_vec()])
    }

    /// Returns the seeds for the PDA used for signing
    pub fn signing_seeds(&self) -> Vec<Vec<u8>> {
        let mut slasher_seeds = Self::seeds(&self.base);
        slasher_seeds.push(vec![self.bump]);
        slasher_seeds
    }

    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the account as an [`Resolver`] account, returning an error if it is not.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `account` - The account to load the NCN operator ticket from
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        account: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if account.owner.ne(program_id) {
            msg!("Slasher account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_is_empty() {
            msg!("Slasher account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !account.is_writable {
            msg!("Slasher account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Slasher account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let base = Self::try_from_slice_unchecked(&account.data.borrow())?.base;
        let expected_pubkey = Self::find_program_address(program_id, &base).0;
        if account.key.ne(&expected_pubkey) {
            msg!("Slasher account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
