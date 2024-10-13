use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{types::PodU64, AccountDeserialize, Discriminator};
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

/// The global configuration account for the resolver program. Manages
/// program-wide settings and state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct NcnResolverProgramConfig {
    /// The length of an epoch in slots
    veto_duration: PodU64,

    resolver_count: PodU64,

    /// The bump seed for the PDA
    pub bump: u8,
}

impl Discriminator for NcnResolverProgramConfig {
    const DISCRIMINATOR: u8 = 1;
}

impl NcnResolverProgramConfig {
    pub fn new(veto_duration: u64, bump: u8) -> Self {
        Self {
            veto_duration: PodU64::from(veto_duration),
            resolver_count: PodU64::from(0),
            bump,
        }
    }

    pub fn veto_duration(&self) -> u64 {
        self.veto_duration.into()
    }

    pub fn resolver_count(&self) -> u64 {
        self.resolver_count.into()
    }

    pub fn increment_resolver_count(&mut self) {
        let mut count = self.resolver_count();
        count += 1;
        self.resolver_count = PodU64::from(count);
    }

    pub fn seeds(ncn: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_resolver_program_config".to_vec(),
            ncn.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(program_id: &Pubkey, ncn: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(ncn);
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
        ncn: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if account.owner.ne(program_id) {
            msg!("NcnResolverProgramConfig account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_is_empty() {
            msg!("NcnResolverProgramConfig account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !account.is_writable {
            msg!("NcnResolverProgramConfig account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("NcnResolverProgramConfig account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let expected_pubkey = Self::find_program_address(program_id, ncn.key).0;
        if account.key.ne(&expected_pubkey) {
            msg!("NcnResolverProgramConfig account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
