use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{AccountDeserialize, Discriminator};
use resolver_sdk::error::ResolverError;
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

/// The vault configuration account for the vault program.
/// Manages program-wide settings and state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct NcnSlashProposalTicket {
    /// The NCN account
    pub ncn: Pubkey,

    pub slash_proposal: Pubkey,

    pub resolver: Pubkey,

    /// The bump seed for the PDA
    pub bump: u8,
    // Reserved space
    // reserved: [u8; 263],
}

impl Discriminator for NcnSlashProposalTicket {
    const DISCRIMINATOR: u8 = 6;
}

impl NcnSlashProposalTicket {
    pub const MAX_SLASH_REQUEST: usize = 32;

    pub fn new(ncn: Pubkey, slash_proposal: Pubkey, bump: u8) -> Self {
        Self {
            ncn,
            slash_proposal,
            resolver: Pubkey::default(),
            bump,
            // reserved: [0; 263],
        }
    }

    pub fn set_resolver(&mut self, new_resolver: Pubkey) {
        self.resolver = new_resolver;
    }

    pub fn check_slash_proposal(&self, slash_proposal: &Pubkey) -> Result<(), ResolverError> {
        if self.slash_proposal.ne(slash_proposal) {
            msg!("Slash proposal is incorrect");
            return Err(ResolverError::SlashProposalInvalid);
        }

        Ok(())
    }

    pub fn check_resolver(&self, resolver: &Pubkey) -> Result<(), ResolverError> {
        if self.resolver.ne(resolver) {
            msg!("Slash proposal's resolver is incorrect");
            return Err(ResolverError::SlashProposalResolverInvalid);
        }

        Ok(())
    }

    pub fn seeds(ncn: &Pubkey, slash_proposal: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_slash_proposal_ticket".to_vec(),
            ncn.as_ref().to_vec(),
            slash_proposal.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        ncn: &Pubkey,
        slash_proposal: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(ncn, slash_proposal);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the account as an [`RequestSlashList`] account, returning an error if it is not.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `slash_request_list` - The account to load the SlashRequestList from
    /// * `ncn` - The account to load the SlashRequestList from
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        ncn_slash_proposal_ticket: &AccountInfo,
        ncn: &AccountInfo,
        slash_proposal: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if ncn_slash_proposal_ticket.owner.ne(program_id) {
            msg!("NcnSlashProposalTicket account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if ncn_slash_proposal_ticket.data_is_empty() {
            msg!("NcnSlashProposalTicket account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !ncn_slash_proposal_ticket.is_writable {
            msg!("NcnSlashProposalTicket account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if ncn_slash_proposal_ticket.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("NcnSlashProposalTicket account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }

        let expected_pubkey = Self::find_program_address(program_id, ncn.key, slash_proposal.key).0;
        if ncn_slash_proposal_ticket.key.ne(&expected_pubkey) {
            msg!("NcnSlashProposalTicket account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
