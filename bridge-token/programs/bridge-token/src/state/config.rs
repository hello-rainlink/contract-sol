use anchor_lang::prelude::*;
use bridge_core::state::config::Chain;

#[account]
pub struct ConfigInfo {
    pub admin: Pubkey,
    pub bump: u8,
    pub padding: [u8; 32],
}
impl ConfigInfo {
    pub const LEN: usize = 32 + 1 + 32;
    pub const SEEDS: &str = super::CONFIG_SEED;
}

#[account]
pub struct TokenRelation {
    pub from_chain: Chain,
    pub from_token: [u8; 32],
    pub from_decimals: u8,
    pub to_token: Pubkey,
    pub to_decimals: u8,
    pub mint_type: u8,
}
impl TokenRelation {
    pub const LEN: usize = Chain::LEN + 32 + 1 + 32 + 1 + 1;
}

#[account]
pub struct ChainRelation {
    pub from_chain: Chain,
    pub from_excutor: [u8; 32],
    pub fee_token: [u8; 32],
}
impl ChainRelation {
    pub const LEN: usize = Chain::LEN + 32 + 32;
}
