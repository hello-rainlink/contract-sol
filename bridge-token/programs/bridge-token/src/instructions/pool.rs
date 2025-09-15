use crate::state::{
    config::ConfigInfo,
    pool::{Lp, Pool, SCALING_FACTOR},
    BRIDGE_SEED,
};
use anchor_lang::{
    prelude::*,
    solana_program::{self, program::invoke},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, spl_token, Mint, Token, TokenAccount},
};

/// New a pool from
pub fn pool_new(ctx: Context<PoolNew>) -> Result<()> {
    let pool_account = &mut ctx.accounts.pool_account;
    pool_account.token_mint = ctx.accounts.token_mint.key();
    pool_account.last_receive_rewards_time = Clock::get()?.unix_timestamp;
    pool_account.pool_fee_rate = 3000; // 0.3%

    Ok(())
}

/// set pool fee rate
pub fn set_pool_fee_rate(ctx: Context<PoolFeeRate>, fee_rate: u64) -> Result<()> {
    if fee_rate > 1_000_000 {
        return Err(ProgramError::InvalidArgument.into());
    }
    let pool_account = &mut ctx.accounts.pool_account;
    pool_account.pool_fee_rate = fee_rate;
    Ok(())
}

/// add liquidity to pool from
pub fn add_liquidity(ctx: Context<PoolLiquidity>, amount: u64) -> Result<()> {
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.payer_token.to_account_info(),
                to: ctx.accounts.fund_pool.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount,
    )?;

    // Change pool
    let pool_account = &mut ctx.accounts.pool_account;
    pool_account.total_staked += amount;
    pool_account.total_staked_liquidity += amount;
    pool_account.total_liquidity += amount;

    // Change lp
    let scaled_debt = amount
        .checked_mul(pool_account.acc_ratio)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let debt_increment = scaled_debt / SCALING_FACTOR;

    let lp_account = &mut ctx.accounts.lp_account;
    lp_account.amount += amount;
    lp_account.debt = lp_account
        .debt
        .checked_add(debt_increment)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    Ok(())
}

/// remove liquidity to pool from
pub fn remove_liquidity(ctx: Context<PoolLiquidity>, amount: u64) -> Result<()> {
    // Change lp
    let lp_account = &mut ctx.accounts.lp_account;
    if lp_account.amount < amount {
        return Err(crate::error::ErrorCode::Illiquidity.into());
    }

    // calculate partial debt
    let old_amount = lp_account.amount;
    let new_reward = amount
        .checked_mul(ctx.accounts.pool_account.acc_ratio)
        .ok_or(ProgramError::ArithmeticOverflow)?
        / SCALING_FACTOR;
    let part_debt = lp_account
        .debt
        .checked_mul(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?
        / old_amount;
    if new_reward > part_debt {
        lp_account.remaining = lp_account
            .remaining
            .checked_add(new_reward - part_debt)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }
    let debt_reduction = part_debt;

    lp_account.amount -= amount;
    lp_account.debt = lp_account
        .debt
        .checked_sub(debt_reduction)
        .ok_or(ProgramError::InvalidArgument)?;

    // Change pool
    let pool_account = &mut ctx.accounts.pool_account;
    if pool_account.total_liquidity < amount {
        return Err(crate::error::ErrorCode::TotalIlliquidity.into());
    }

    let staked_decrease = calc_staked_decrease(
        old_amount,
        amount,
        pool_account.total_staked_liquidity,
        pool_account.total_staked,
    );
    if staked_decrease > pool_account.total_staked {
        return Err(crate::error::ErrorCode::StakedDecreaseTooLarge.into());
    }
    pool_account.total_staked -= staked_decrease;
    pool_account.total_staked_liquidity -= amount;
    pool_account.total_liquidity -= amount;

    let signer_seeds: &[&[&[u8]]] = &[&[BRIDGE_SEED.as_bytes(), &[ctx.bumps.bridge_authority]]];
    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.fund_pool.to_account_info(),
                to: ctx.accounts.payer_token.to_account_info(),
                authority: ctx.accounts.bridge_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;
    Ok(())
}

pub fn withdrawal(ctx: Context<PoolWithdrawal>, amount: u64) -> Result<()> {
    let pool_account = &mut ctx.accounts.pool_account;
    let lp_account = &mut ctx.accounts.lp_account;

    let scaled_amount = lp_account
        .amount
        .checked_mul(pool_account.acc_ratio)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let amount_ratio = scaled_amount / SCALING_FACTOR;
    let reward = amount_ratio
        .checked_sub(lp_account.debt)
        .ok_or(ProgramError::InvalidArgument)?
        .checked_add(lp_account.remaining)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    require!(
        reward >= amount,
        crate::error::ErrorCode::WithdrawalAmountExceedReward
    );
    pool_account.total_liquidity -= amount;

    let seeds = &[BRIDGE_SEED.as_bytes(), &[ctx.bumps.bridge_authority]];
    let signer_seeds = &[&seeds[..]];
    // transfer reward
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Transfer {
            from: ctx.accounts.fund_pool.to_account_info(),
            to: ctx.accounts.payer_token.to_account_info(),
            authority: ctx.accounts.bridge_authority.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(cpi_ctx, amount)?;

    if ctx.accounts.token_mint.key() == spl_token::native_mint::ID {
        // wsol to sol
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: ctx.accounts.payer_token.to_account_info(),
                destination: ctx.accounts.payer.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        );
        token::close_account(cpi_ctx)?;
    }

    // update the remaining
    lp_account.earns += amount;
    lp_account.debt = lp_account.amount * pool_account.acc_ratio;
    lp_account.remaining = reward - amount;
    Ok(())
}

fn calc_staked_decrease(
    old_amount: u64,
    amount: u64,
    total_staked_liquidity: u64,
    total_staked: u64,
) -> u64 {
    let precision: u128 = 1_000_000_000_000; // Magnify 1e12 processing accuracy

    let user_stake_ratio = (old_amount as u128) * precision / (total_staked_liquidity as u128);
    let withdraw_ratio = (amount as u128) * precision / (old_amount as u128);

    let mut staked_decrease = (total_staked as u128) * user_stake_ratio / precision;
    staked_decrease = (staked_decrease * withdraw_ratio) / precision;

    staked_decrease as u64
}

/// the instruction must same to params
#[derive(Accounts)]
pub struct PoolNew<'info> {
    #[account(mut, constraint = payer.key() == bridge_config.admin || payer.key() == crate::ID)]
    pub payer: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(init_if_needed, payer = payer, associated_token::mint = token_mint, associated_token::authority = bridge_authority)]
    pub fund_pool: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed, payer = payer, seeds = [token_mint.key().as_ref(), Pool::SEEDS.as_bytes()], bump, space = 8 + Pool::LEN)]
    pub pool_account: Account<'info, Pool>,
    /// CHECK:
    #[account(seeds = [BRIDGE_SEED.as_bytes()], bump)]
    pub bridge_authority: AccountInfo<'info>,
    #[account(seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct PoolFeeRate<'info> {
    #[account(mut, constraint = payer.key() == bridge_config.admin || payer.key() == crate::ID)]
    pub payer: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
    #[account(mut, seeds = [token_mint.key().as_ref(), Pool::SEEDS.as_bytes()], bump)]
    pub pool_account: Account<'info, Pool>,
}

/// the instruction must same to params
#[derive(Accounts)]
pub struct PoolLiquidity<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut, associated_token::mint = token_mint, associated_token::authority = payer)]
    pub payer_token: Box<Account<'info, TokenAccount>>,
    #[account(mut, associated_token::mint = token_mint, associated_token::authority = bridge_authority)]
    pub fund_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [token_mint.key().as_ref(), Pool::SEEDS.as_bytes()], bump)]
    pub pool_account: Account<'info, Pool>,
    #[account(init_if_needed, payer = payer, seeds = [pool_account.key().as_ref(), payer.key().as_ref(), Lp::SEEDS.as_bytes()], bump, space = 8 + Lp::LEN)]
    pub lp_account: Account<'info, Lp>,
    /// CHECK:
    #[account(seeds = [BRIDGE_SEED.as_bytes()], bump)]
    pub bridge_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// the instruction must same to params
#[derive(Accounts)]
pub struct PoolWithdrawal<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(init_if_needed, payer = payer, associated_token::mint = token_mint, associated_token::authority = payer)]
    pub payer_token: Box<Account<'info, TokenAccount>>,
    #[account(mut, associated_token::mint = token_mint, associated_token::authority = bridge_authority)]
    pub fund_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [token_mint.key().as_ref(), Pool::SEEDS.as_bytes()], bump)]
    pub pool_account: Account<'info, Pool>,
    #[account(mut, seeds = [pool_account.key().as_ref(), payer.key().as_ref(), Lp::SEEDS.as_bytes()], bump)]
    pub lp_account: Account<'info, Lp>,
    /// CHECK:
    #[account(seeds = [BRIDGE_SEED.as_bytes()], bump)]
    pub bridge_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
