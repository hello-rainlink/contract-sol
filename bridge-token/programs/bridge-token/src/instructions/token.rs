use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata,
    },
    token::{Mint, Token},
};

use crate::state::{config::ConfigInfo, BRIDGE_SEED};

/// New a token from
pub fn token_new(
    ctx: Context<TokenNew>,
    _decimals: u8,
    _symbol: String,
    _name: String,
    _uri: String,
) -> Result<()> {
    let signer_seeds: &[&[&[u8]]] = &[&[BRIDGE_SEED.as_bytes(), &[ctx.bumps.bridge_authority]]];
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                mint_authority: ctx.accounts.bridge_authority.to_account_info(),
                update_authority: ctx.accounts.bridge_authority.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds
        ),
        DataV2 {
            name: _name,
            symbol: _symbol,
            uri: _uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        true,
        true,
        None,
    )?;
    Ok(())
}

/// the instruction must same to params
#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct TokenNew<'info> {
    // #[account(mut, constraint = payer.key() == bridge_config.admin || payer.key() == crate::ID)]
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, mint::decimals = decimals, mint::authority = bridge_authority)]
    pub token_mint: Account<'info, Mint>,
    /// CHECK:
    #[account(seeds = [BRIDGE_SEED.as_bytes()], bump)]
    pub bridge_authority: AccountInfo<'info>,
    #[account(seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
    /// CHECK
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
