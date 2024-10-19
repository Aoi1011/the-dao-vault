mod execute_slash;
mod initialize_config;
mod initialize_ncn_resolver_program_config;
mod initialize_resolver;
mod initialize_slasher;
mod propose_slash;
mod set_resolver;
mod veto_slash;

pub(crate) const MINT_AMOUNT: u64 = 100_000;
pub(crate) const DELEGATION_AMOUNT: u64 = 10_000;
pub(crate) const MAX_SLASH_AMOUNT: u64 = 100;
pub(crate) const VETO_DURATION: u64 = 100;
