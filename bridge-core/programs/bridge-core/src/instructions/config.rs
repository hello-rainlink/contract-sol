use anchor_lang::prelude::*;

use crate::state::config::ConfigInfo;

pub fn initialize(ctx: Context<ConfInitialize>, bump: u8) -> Result<()> {
    let bridge_config = &mut ctx.accounts.bridge_config;
    bridge_config.admin = ctx.accounts.authority.key();
    bridge_config.threshold = 1;
    bridge_config.bump = bump;
    Ok(())
}

pub fn change_admin(ctx: Context<BridgeConf>, new_admin: Pubkey) -> Result<()> {
    ctx.accounts.bridge_config.admin = new_admin;
    Ok(())
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ConfInitialize<'info> {
    #[account(init, payer = authority, seeds = [ConfigInfo::SEEDS.as_bytes()], bump, space = 8 + ConfigInfo::LEN)]
    pub bridge_config: Account<'info, ConfigInfo>,
    /// CHECK
    #[account(init, payer = authority, seeds = [crate::MESSAGE_FEE_SEED.as_bytes()], bump, space = 0)]
    pub message_fee: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BridgeConf<'info> {
    #[account(constraint = admin.key() == bridge_config.admin || admin.key() == crate::ID)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
}
