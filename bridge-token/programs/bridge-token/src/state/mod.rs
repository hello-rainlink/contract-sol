use anchor_lang::constant;

pub mod config;
pub mod executor;
pub mod pool;

#[constant]
pub const BRIDGE_SEED: &str = "bridge";
#[constant]
pub const CONFIG_SEED: &str = "global";
#[constant]
pub const CHAIN_RELATION_SEED: &str = "chain_relation";
#[constant]
pub const LP_SEED: &str = "lp";
#[constant]
pub const POOL_SEED: &str = "pool";

pub enum MintType {
    Mint,
    Lp,
}
