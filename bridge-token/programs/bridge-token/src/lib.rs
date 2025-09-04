use crate::state::executor::MsgBody;
use anchor_lang::prelude::*;
use bridge_core::state::config::Chain;
use bridge_core::state::config::MsgHeader;
use instructions::{
    config::{ConfInitialize, *},
    executor::*,
    pool::{PoolNew, *},
    token::{TokenNew, *},
    *,
};

mod error;
mod instructions;
pub mod state;

declare_id!("C7JbQuFuapFwBK7oCunFyN8zSi5Hmfgrq1LZzrmiago3");

#[program]
pub mod bridge_token {

    use super::*;

    pub fn initialize(ctx: Context<ConfInitialize>, bump: u8) -> Result<()> {
        config::initialize(ctx, bump)?;
        Ok(())
    }

    pub fn change_admin(ctx: Context<BridgeConf>, new_admin: Pubkey) -> Result<()> {
        config::change_admin(ctx, new_admin)?;
        Ok(())
    }

    pub fn token_relationship(
        ctx: Context<TokenRelationship>,
        from_chain: Chain,
        from_token: [u8; 32],
        from_decimals: u8,
        mint_type: u8,
    ) -> Result<()> {
        config::token_relationship(ctx, from_chain, from_token, from_decimals, mint_type)?;
        Ok(())
    }

    pub fn token_relationship_remove(
        ctx: Context<TokenRelationshipRemove>,
        from_chain: Chain,
        from_token: [u8; 32],
    ) -> Result<()> {
        config::token_relationship_remove(ctx, from_chain, from_token)?;
        Ok(())
    }

    pub fn chain_relationship(
        ctx: Context<ChainRelationship>,
        from_chain: Chain,
        executor: Option<[u8; 32]>,
        fee_token: Option<[u8; 32]>,
    ) -> Result<()> {
        config::chain_relationship(ctx, from_chain, executor, fee_token)?;
        Ok(())
    }

    pub fn token_new(
        ctx: Context<TokenNew>,
        decimals: u8,
        symbol: String,
        name: String,
        uri: String,
    ) -> Result<()> {
        token::token_new(ctx, decimals, symbol, name, uri)?;
        Ok(())
    }

    pub fn set_pool_fee_rate(ctx: Context<PoolFeeRate>, fee_rate: u64) -> Result<()> {
        pool::set_pool_fee_rate(ctx, fee_rate)?;
        Ok(())
    }

    pub fn pool_new(ctx: Context<PoolNew>) -> Result<()> {
        pool::pool_new(ctx)?;
        Ok(())
    }

    pub fn add_liquidity(ctx: Context<PoolLiquidity>, amount: u64) -> Result<()> {
        pool::add_liquidity(ctx, amount)?;
        Ok(())
    }

    pub fn remove_liquidity(ctx: Context<PoolLiquidity>, amount: u64) -> Result<()> {
        pool::remove_liquidity(ctx, amount)?;
        Ok(())
    }

    pub fn withdrawal(ctx: Context<PoolWithdrawal>, amount: u64) -> Result<()> {
        pool::withdrawal(ctx, amount)?;
        Ok(())
    }

    pub fn bridge_proposal(
        ctx: Context<Proposal>,
        to_chain: Chain,
        to_token: [u8; 32],
        to_who: [u8; 32],
        all_amount: u64,
        upload_gas_fee: u64,
    ) -> Result<()> {
        executor::bridge_proposal(ctx, to_chain, to_token, to_who, all_amount, upload_gas_fee)?;
        Ok(())
    }

    pub fn bridge_finish(
        ctx: Context<Consumption>,
        msg_header: MsgHeader,
        msg_body: MsgBody,
        accum_pk: Vec<u8>,
        signatures: Vec<[u8; 65]>,
    ) -> Result<()> {
        executor::bridge_finish(ctx, msg_header, msg_body, accum_pk, signatures)?;
        Ok(())
    }
}
