use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{types::PodU64, AccountDeserialize, Discriminator};
use resolver_sdk::error::ResolverError;
use shank::{ShankAccount, ShankType};
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

#[derive(Debug, Clone, Copy, Zeroable, ShankType, Pod, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct Resolver {
    /// The base pubkey used as a seed for the PDA
    pub base: Pubkey,

    /// The admin pubkey
    pub admin: Pubkey,

    /// The resolver index
    index: PodU64,

    /// The bump seed for the PDA
    pub bump: u8,
}

impl Discriminator for Resolver {
    const DISCRIMINATOR: u8 = 2;
}

impl Resolver {
    pub fn new(base: Pubkey, admin: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            base,
            admin,
            index: PodU64::from(index),
            bump,
        }
    }

    pub fn index(&self) -> u64 {
        self.index.into()
    }

    pub fn check_admin(&self, candidate_resolver_admin: &Pubkey) -> Result<(), ResolverError> {
        if self.admin.ne(candidate_resolver_admin) {
            msg!("Resolver admin is incorrect");
            return Err(ResolverError::ResolverAdminInvalid);
        }

        Ok(())
    }

    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"resolver".to_vec(), base.as_ref().to_vec()])
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
            msg!("Resolver account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_is_empty() {
            msg!("Resolver account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !account.is_writable {
            msg!("Resolver account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Resolver account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let base = Self::try_from_slice_unchecked(&account.data.borrow())?.base;
        let expected_pubkey = Self::find_program_address(program_id, &base).0;
        if account.key.ne(&expected_pubkey) {
            msg!("Resolver account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
