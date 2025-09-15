use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use bridge_core::state::config::Chain;

use crate::state::{
    config::{ChainRelation, ConfigInfo, TokenRelation}, BRIDGE_SEED, CHAIN_RELATION_SEED
};

pub fn initialize(ctx: Context<ConfInitialize>, bump: u8) -> Result<()> {
    let bridge_config = &mut ctx.accounts.bridge_config;
    bridge_config.admin = ctx.accounts.authority.key();
    bridge_config.bump = bump;
    Ok(())
}

pub fn change_admin(ctx: Context<BridgeConf>, new_admin: Pubkey) -> Result<()> {
    ctx.accounts.bridge_config.admin = new_admin;
    Ok(())
}

pub fn token_relationship(
    ctx: Context<TokenRelationship>,
    from_chain: Chain,
    from_token: [u8; 32],
    from_decimals: u8,
    mint_type: u8,
) -> Result<()> {
    let token_relation = &mut ctx.accounts.token_relation;
    token_relation.from_chain = from_chain;
    token_relation.from_token = from_token
        .try_into()
        .map_err(|_| crate::error::ErrorCode::ConversionError)?;
    token_relation.from_decimals = from_decimals;
    token_relation.to_token = ctx.accounts.token_mint.key();
    token_relation.to_decimals = ctx.accounts.token_mint.decimals;
    //todo If this method is opened, except for admin, mint_type can only be mint
    token_relation.mint_type = mint_type;
    Ok(())
}

pub fn token_relationship_remove(
    ctx: Context<TokenRelationshipRemove>,
    _from_chain: Chain,
    _from_token: [u8; 32],
) -> Result<()> {
    let token_relation = &mut ctx.accounts.token_relation;
    require!(token_relation.to_token == ctx.accounts.token_mint.key(), crate::error::ErrorCode::TokenRelationNotFound);
    **ctx.accounts.admin.to_account_info().lamports.borrow_mut() += token_relation.to_account_info().lamports();
    **token_relation.to_account_info().lamports.borrow_mut() = 0;
    Ok(())
}

pub fn chain_relationship(
    ctx: Context<ChainRelationship>,
    from_chain: Chain,
    executor: Option<[u8; 32]>,
    fee_token: Option<[u8; 32]>,
) -> Result<()> {
    let chain_relation = &mut ctx.accounts.chain_relation;
    chain_relation.from_chain = from_chain;
    if let Some(executor) = executor {
        chain_relation.from_excutor = executor
            .try_into()
            .map_err(|_| crate::error::ErrorCode::ConversionError)?;
    }
    if let Some(fee_token) = fee_token {
        chain_relation.fee_token = fee_token
            .try_into()
            .map_err(|_| crate::error::ErrorCode::ConversionError)?;
    }
    Ok(())
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ConfInitialize<'info> {
    #[account(init_if_needed, payer = authority, seeds = [ConfigInfo::SEEDS.as_bytes()], bump, space = 8 + ConfigInfo::LEN)]
    pub bridge_config: Account<'info, ConfigInfo>,
    #[account(mut)]
    pub authority: Signer<'info>,
       /// CHECK:
    #[account(init, payer = authority, seeds = [BRIDGE_SEED.as_bytes()], bump, space = 0)]
    pub bridge_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BridgeConf<'info> {
    #[account(constraint = admin.key() == bridge_config.admin || admin.key() == crate::ID)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
}

#[derive(Accounts)]
#[instruction(from_chain: Chain, from_token: [u8; 32])]
pub struct TokenRelationship<'info> {
    #[account(mut, constraint = admin.key() == bridge_config.admin || admin.key() == crate::ID)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
    pub token_mint: Account<'info, Mint>,
    #[account(init_if_needed, payer = admin, seeds = [&from_chain.combain_chain(), &from_token.to_vec()], bump, space = 8 + TokenRelation::LEN)]
    pub token_relation: Account<'info, TokenRelation>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(from_chain: Chain, from_token: [u8; 32])]
pub struct TokenRelationshipRemove<'info> {
    #[account(mut, constraint = admin.key() == bridge_config.admin || admin.key() == crate::ID)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut, seeds = [&from_chain.combain_chain(), &from_token.to_vec()], bump, close = admin)]
    pub token_relation: Account<'info, TokenRelation>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(from_chain:Chain)]
pub struct ChainRelationship<'info> {
    #[account(mut, constraint = admin.key() == bridge_config.admin || admin.key() == crate::ID)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
    #[account(init_if_needed, payer = admin, seeds = [&from_chain.combain_chain(), CHAIN_RELATION_SEED.as_bytes()], bump, space = 8 + ChainRelation::LEN)]
    pub chain_relation: Account<'info, ChainRelation>,
    pub system_program: Program<'info, System>,
}
