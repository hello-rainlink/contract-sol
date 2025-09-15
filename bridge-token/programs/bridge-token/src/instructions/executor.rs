use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, system_instruction},
};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, spl_token, Mint, MintTo, Token, TokenAccount},
};
use bridge_core::state::config::{Chain, ConfigInfo, MsgHeader, ToChainNonce, MESSAGE_FEE_SEED};

use crate::state::{
    config::{ChainRelation, TokenRelation},
    executor::MsgBody,
    pool::Pool,
    MintType, BRIDGE_SEED, CHAIN_RELATION_SEED,
};

pub fn bridge_proposal(
    ctx: Context<Proposal>,
    to_chain: Chain,
    _to_token: [u8; 32],
    to_who: [u8; 32],
    all_amount: u64,
    upload_gas_fee: u64,
) -> Result<()> {
    // transfer gas fee to pool
    invoke(
        &system_instruction::transfer(
            ctx.accounts.sender.key,
            &ctx.accounts.bridge_authority.key(),
            upload_gas_fee,
        ),
        &[
            ctx.accounts.sender.to_account_info(),
            ctx.accounts.bridge_authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;
    let seeds = &[BRIDGE_SEED.as_bytes(), &[ctx.bumps.bridge_authority]];
    let signer_seeds = &[&seeds[..]];
    // transfer or burn all_amount
    let token_relation = &ctx.accounts.token_relation;
    if token_relation.mint_type == MintType::Mint as u8 {
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.sender_token.to_account_info(),
                    to: ctx.accounts.fund_pool.to_account_info(),
                    authority: ctx.accounts.sender.to_account_info(),
                },
            ),
            all_amount,
        )?;
        anchor_spl::token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: ctx.accounts.token_mint.to_account_info(),
                    from: ctx.accounts.fund_pool.to_account_info(),
                    authority: ctx.accounts.bridge_authority.to_account_info(),
                },
                signer_seeds,
            ),
            all_amount,
        )?;
    } else {
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.sender_token.to_account_info(),
                    to: ctx.accounts.fund_pool.to_account_info(),
                    authority: ctx.accounts.sender.to_account_info(),
                },
            ),
            all_amount,
        )?;
        ctx.accounts
            .pool_account
            .transfer_to_pool(all_amount as i64)?;
    }

    // generate message body
    let msg_body = MsgBody {
        source_token: ctx.accounts.token_mint.key().to_bytes(),
        all_amount: all_amount as u128,
        from_who: ctx.accounts.sender.key().to_bytes(),
        to_who: to_who
            .try_into()
            .map_err(|_| crate::error::ErrorCode::ConversionError)?,
    };

    let cpi_program = ctx.accounts.bridge_core_program.to_account_info();
    let cpi_accounts = bridge_core::cpi::accounts::SendToOtherChain {
        to_chain_nonce_account: ctx.accounts.to_chain_nonce_account.to_account_info(),
        sender: ctx.accounts.sender.to_account_info(),
        message_fee: ctx.accounts.message_fee.to_account_info(),
        bridge_config: ctx.accounts.bridge_config.to_account_info(),
        caller_auth_pda: ctx.accounts.bridge_authority.to_account_info(),
        caller_program: ctx.accounts.program_id.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    bridge_core::cpi::send_message(
        cpi_ctx,
        to_chain,
        ctx.accounts.chain_relation.from_excutor,
        msg_body.to_evm_buffer(),
        0,
        upload_gas_fee,
    )?;

    Ok(())
}

pub fn bridge_finish(
    ctx: Context<Consumption>,
    msg_header: MsgHeader,
    msg_body: MsgBody,
    accum_pk: Vec<u8>,
    signatures: Vec<[u8; 65]>,
) -> Result<()> {
    require!(
        msg_header.from_addr == ctx.accounts.chain_relation.from_excutor,
        crate::error::ErrorCode::SenderAddrNotMatch
    );

    require!(
        msg_header.to_addr == crate::ID.to_bytes(),
        crate::error::ErrorCode::ExecuteAddrNotMatch
    );

    // verify msg
    let cpi_program = ctx.accounts.bridge_core_program.to_account_info();
    let cpi_accounts = bridge_core::cpi::accounts::ConfirmFromOtherChain {
        from_chain_nonce_account: ctx.accounts.from_chain_nonce_account.to_account_info(),
        bridge_config: ctx.accounts.bridge_config.to_account_info(),
        user: ctx.accounts.sender.to_account_info(),
        receiver: ctx.accounts.receiver.to_account_info(),
        caller_auth_pda: ctx.accounts.bridge_authority.to_account_info(),
        caller_program: ctx.accounts.program_id.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let seeds = &[BRIDGE_SEED.as_bytes(), &[ctx.bumps.bridge_authority]];
    let signer_seeds = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    bridge_core::cpi::confirm_message(
        cpi_ctx,
        msg_header.clone(),
        msg_body.to_evm_buffer(),
        accum_pk,
        signatures,
    )?;

    // decode the body
    if ctx.accounts.token_relation.mint_type == MintType::Mint as u8 {
        let from_decimals = ctx.accounts.token_relation.from_decimals;
        let to_decimals = ctx.accounts.token_relation.to_decimals;
        let all_amount = msg_body
            .all_amount
            .checked_mul(10u128.pow(to_decimals.into()))
            .and_then(|v| v.checked_div(10u128.pow(from_decimals.into())))
            .expect("overflow in decimals conversion");
        msg!("mint token==> {}", all_amount);
        // mint token
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.receiver_token_account.to_account_info(),
                authority: ctx.accounts.bridge_authority.to_account_info(),
            },
            signer_seeds,
        );
        token::mint_to(cpi_ctx, all_amount as u64)?;
    } else {
        let from_decimals = ctx.accounts.token_relation.from_decimals;
        let to_decimals = ctx.accounts.token_relation.to_decimals;
        let all_amount =
            msg_body.all_amount * 10u128.pow(to_decimals.into()) / 10u128.pow(from_decimals.into());

        let lp_fee = all_amount * ctx.accounts.pool_account.pool_fee_rate as u128 / 1000000;
        let final_amount = all_amount - lp_fee;
        msg!("transfer token==> {} lp_fee {}", final_amount, lp_fee);
        // calc fee to lp provider
        let balance = ctx.accounts.fund_pool.amount;
        ctx.accounts
            .pool_account
            .refresh_rewards(balance, lp_fee as u64, all_amount as u64)?;

        // transfer token
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.fund_pool.to_account_info(),
                to: ctx.accounts.receiver_token_account.to_account_info(),
                authority: ctx.accounts.bridge_authority.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx, final_amount as u64)?;
        if ctx.accounts.token_mint.key() == spl_token::native_mint::ID {
            // wsol to sol
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::CloseAccount {
                    account: ctx.accounts.receiver_token_account.to_account_info(),
                    destination: ctx.accounts.sender.to_account_info(),
                    authority: ctx.accounts.sender.to_account_info(),
                },
            );
            token::close_account(cpi_ctx)?;

            // transfer sol to receiver
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.sender.key,
                    &ctx.accounts.receiver.key(),
                    final_amount as u64,
                ),
                &[
                    ctx.accounts.sender.to_account_info(),
                    ctx.accounts.receiver.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }
    }

    // mint or transfer gas fee to sender
    if ctx.accounts.fee_token_relation.mint_type == MintType::Mint as u8 {
        let from_decimals = ctx.accounts.fee_token_relation.from_decimals;
        let to_decimals = ctx.accounts.fee_token_relation.to_decimals;
        let gas_fee = msg_header
            .upload_gas_fee
            .checked_mul(10u128.pow(to_decimals.into()))
            .and_then(|v| v.checked_div(10u128.pow(from_decimals.into())))
            .expect("overflow in decimals conversion");
        // mint token
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.fee_token_mint.to_account_info(),
                to: ctx.accounts.sender_fee_token_account.to_account_info(),
                authority: ctx.accounts.bridge_authority.to_account_info(),
            },
            signer_seeds,
        );
        token::mint_to(cpi_ctx, gas_fee as u64)?;
    } else {
        let from_decimals = ctx.accounts.fee_token_relation.from_decimals;
        let to_decimals = ctx.accounts.fee_token_relation.to_decimals;
        let gas_fee = msg_header.upload_gas_fee * 10u128.pow(to_decimals.into())
            / 10u128.pow(from_decimals.into());
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.fee_fund_pool.to_account_info(),
                to: ctx.accounts.sender_fee_token_account.to_account_info(),
                authority: ctx.accounts.bridge_authority.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx, gas_fee as u64)?;

        // transfer gas fee from pool
        ctx.accounts
            .fee_pool_account
            .transfer_from_pool(gas_fee as i64)?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct EmptyAccounts {}

#[derive(Accounts)]
#[instruction(to_chain:Chain, to_token:[u8;32])]
pub struct Proposal<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    #[account(mut, associated_token::mint = token_mint, associated_token::authority = sender)]
    pub sender_token: Box<Account<'info, TokenAccount>>,
    #[account(mut, associated_token::mint = token_mint, associated_token::authority = bridge_authority)]
    pub fund_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [token_mint.key().as_ref(), Pool::SEEDS.as_bytes()], bump)]
    pub pool_account: Account<'info, Pool>,
    /// CHECK:
    #[account(mut, seeds = [BRIDGE_SEED.as_bytes()], bump)]
    pub bridge_authority: AccountInfo<'info>,
    #[account(seeds = [&to_chain.combain_chain(), &to_token], bump)]
    pub token_relation: Account<'info, TokenRelation>,
    #[account(seeds = [&to_chain.combain_chain(), CHAIN_RELATION_SEED.as_bytes()], bump)]
    pub chain_relation: Account<'info, ChainRelation>,
    /// CHECK:
    #[account(mut, seeds = [&to_chain.combain_chain(), ToChainNonce::SEED_SUFFIX.as_bytes()], bump, seeds::program = bridge_core_program.key())]
    pub to_chain_nonce_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, seeds = [MESSAGE_FEE_SEED.as_bytes()], bump, seeds::program = bridge_core_program.key())]
    pub message_fee: AccountInfo<'info>,
    /// CHECK:
    #[account(seeds = [ConfigInfo::SEEDS.as_bytes()], bump, seeds::program = bridge_core_program.key())]
    pub bridge_config: AccountInfo<'info>,
    pub bridge_core_program: Program<'info, bridge_core::program::BridgeCore>,
    /// CHECK:
    #[account(address = crate::ID)]
    pub program_id: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(msg_header: MsgHeader, msg_body: MsgBody)]
pub struct Consumption<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(mut)]
    pub token_mint: Box<Account<'info, Mint>>,
    /// CHECK:
    #[account(mut, address = Pubkey::new_from_array(msg_body.to_who))]
    pub receiver: AccountInfo<'info>,
    #[account(mut, associated_token::mint = token_mint, associated_token::authority = if token_mint.key() == spl_token::native_mint::ID { sender.to_account_info() } else { receiver.to_account_info() })]
    pub receiver_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK:
    #[account(seeds = [BRIDGE_SEED.as_ref()], bump)]
    pub bridge_authority: AccountInfo<'info>,
    #[account(mut, associated_token::mint = token_mint, associated_token::authority = bridge_authority)]
    pub fund_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [token_mint.key().as_ref(), Pool::SEEDS.as_bytes()], bump)]
    pub pool_account: Box<Account<'info, Pool>>,
    #[account(seeds = [&msg_header.from_chain.combain_chain(), &msg_body.source_token], bump)]
    pub token_relation: Box<Account<'info, TokenRelation>>,
    #[account(seeds = [&msg_header.from_chain.combain_chain(), CHAIN_RELATION_SEED.as_bytes()], bump)]
    pub chain_relation: Box<Account<'info, ChainRelation>>,

    #[account(seeds = [&&msg_header.from_chain.combain_chain(), &chain_relation.fee_token], bump)]
    pub fee_token_relation: Box<Account<'info, TokenRelation>>,
    #[account(mut, address = fee_token_relation.to_token)]
    pub fee_token_mint: Box<Account<'info, Mint>>,
    #[account(mut, associated_token::mint = fee_token_mint, associated_token::authority = bridge_authority)]
    pub fee_fund_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [fee_token_mint.key().as_ref(), Pool::SEEDS.as_bytes()], bump)]
    pub fee_pool_account: Box<Account<'info, Pool>>,
    #[account(mut, associated_token::mint = fee_token_mint, associated_token::authority = sender)]
    pub sender_fee_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK:
    #[account(mut)]
    pub from_chain_nonce_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, seeds = [MESSAGE_FEE_SEED.as_bytes()], bump, seeds::program = bridge_core_program.key())]
    pub message_fee: AccountInfo<'info>,
    /// CHECK:
    #[account(seeds = [ConfigInfo::SEEDS.as_bytes()], bump, seeds::program = bridge_core_program.key())]
    pub bridge_config: AccountInfo<'info>,
    pub bridge_core_program: Program<'info, bridge_core::program::BridgeCore>,
    /// CHECK:
    #[account(address = crate::ID)]
    pub program_id: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
