// use bytemuck::{Pod, Zeroable};
// use jito_bytemuck::{types::PodU16, AccountDeserialize, Discriminator};
// use shank::{ShankAccount, ShankType};
// use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};
//
// #[derive(Debug, Clone, Copy, Zeroable, ShankType, Pod, AccountDeserialize, ShankAccount)]
// #[repr(C)]
// pub struct ResolverNcnTicket {
//     /// The resolver pubkey
//     pub resolver: Pubkey,
//
//     /// The NCN account
//     pub ncn: Pubkey,
//
//     /// The bump seed for the PDA
//     pub bump: u8,
// }
//
// impl Discriminator for ResolverNcnTicket {
//     const DISCRIMINATOR: u8 = 3;
// }
