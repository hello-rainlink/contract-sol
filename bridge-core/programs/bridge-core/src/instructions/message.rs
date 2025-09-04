use std::collections::HashSet;

use super::config::BridgeConf;
use crate::{
    error::ErrorCode,
    evnet::{ConfirmMessage, SendMessage},
    state::config::{
        Chain, ChainType, ConfigInfo, FromChainNonce, MsgHeader, ToChainNonce, CHAIN_ID,
        MESSAGE_FEE_SEED,
    },
};
use anchor_lang::{
    prelude::*,
    solana_program::{
        keccak::hash, program::invoke, secp256k1_recover::secp256k1_recover, system_instruction,
    },
};

/// Initialize the to chain nonce
pub fn init_to_chain_nonce_account(
    _ctx: Context<InitSendToChainNonce>,
    _to_chain: Chain,
) -> Result<()> {
    Ok(())
}

/// Set the bridge fee
pub fn set_bridge_fee(ctx: Context<BridgeConf>, bridge_fee: u64) -> Result<()> {
    ctx.accounts.bridge_config.bridge_fee = bridge_fee;
    Ok(())
}

/// withdraw the bridge fee
pub fn withdraw_fee(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let message_fee_account = &mut ctx.accounts.message_fee;
    if message_fee_account.lamports() >= amount {
        ctx.accounts.message_fee.sub_lamports(amount)?;
        ctx.accounts.super_admin.add_lamports(amount)?;
    } else {
        return Err(ErrorCode::InsufficientFee.into());
    }
    Ok(())
}

/// Send a message to another chain
pub fn send_message(
    ctx: Context<SendToOtherChain>,
    to_chain: Chain,
    to_addr: [u8; 32],
    mbody: Vec<u8>,
    mtype: u8,
    upload_fee: u64,
) -> Result<()> {
    // transfer bridge fee to message fee account
    let bridge_fee = ctx.accounts.bridge_config.bridge_fee;
    invoke(
        &system_instruction::transfer(
            ctx.accounts.sender.key,
            &ctx.accounts.message_fee.key(),
            bridge_fee,
        ),
        &[
            ctx.accounts.sender.to_account_info(),
            ctx.accounts.message_fee.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // update message nonce
    let to_chain_nonce_account = &mut ctx.accounts.to_chain_nonce_account;
    if to_chain_nonce_account.max_nonce == 0 {
        to_chain_nonce_account.chain = to_chain.clone();
    }
    to_chain_nonce_account.max_nonce += 1;

    // Generate the message header
    let mheader = MsgHeader {
        mtype: mtype,
        nonce: to_chain_nonce_account.max_nonce,
        from_chain: get_current_chain(),
        from_addr: ctx.accounts.caller_program.key().to_bytes(),
        to_chain: to_chain,
        to_addr: to_addr,
        upload_gas_fee: upload_fee as u128,
    };

    // Emit the message
    emit!(SendMessage {
        header: mheader,
        body: mbody,
        fee: upload_fee
    });

    Ok(())
}

/// Confirm a message of another chain
pub fn confirm_message(
    ctx: Context<ConfirmFromOtherChain>,
    msg_header: MsgHeader,
    msg_body: Vec<u8>,
    _accum_pk: Vec<u8>,
    signatures: Vec<[u8; 65]>,
) -> Result<()> {
    // check bridge token message
    if msg_body.len() >= 112 {
        let to_who_bytes = &msg_body[80..112];
        require!(
            to_who_bytes == ctx.accounts.receiver.key().to_bytes(),
            crate::error::ErrorCode::ReceiverMismatch
        );
    }

    let validators = ctx.accounts.bridge_config.signers.clone();
    let mut message = msg_header.to_be_bytes();
    message.extend(msg_body.clone());

    let threshold = ctx.accounts.bridge_config.threshold;
    verify_multisig(message, signatures, validators, threshold)?;

    // update message nonce
    let from_chain_nonce_account = &mut ctx.accounts.from_chain_nonce_account;
    if from_chain_nonce_account.last_nonce == 0 {
        from_chain_nonce_account.chain = msg_header.from_chain.clone();
    }

    msg!(
        "msg_header.nonce {}, from_chain_nonce_account.last_nonce {}",
        msg_header.nonce,
        from_chain_nonce_account.last_nonce
    );
    // check if the message is already confirmed
    require!(
        from_chain_nonce_account.last_nonce != msg_header.nonce,
        crate::error::ErrorCode::NonceConsumed
    );
    from_chain_nonce_account.check_and_store_nonce(msg_header.nonce)?;
    from_chain_nonce_account.last_nonce = msg_header.nonce;

    emit!(ConfirmMessage {
        executor: ctx.accounts.user.key(),
        from_chain: msg_header.from_chain,
        nonce: msg_header.nonce,
        mbody: msg_body.clone(),
    });
    Ok(())
}

/// Verify a multisig message of another chain
fn verify_multisig(
    message: Vec<u8>,
    signatures: Vec<[u8; 65]>,
    validators: Vec<[u8; 20]>,
    threshold: u8,
) -> Result<()> {
    let mut signers = HashSet::new();
    // check every validator
    for signature in signatures.iter() {
        let recovered_pubkey = secp256k1_recover(
            &hash(&message).to_bytes(),
            (signature[64] + 1) % 2,
            &signature[..64],
        )
        .unwrap();

        let hash = hash(&recovered_pubkey.0).to_bytes();
        let mut validator = [0u8; 20];
        validator.copy_from_slice(&hash[12..]);

        require!(
            validators.contains(&validator),
            crate::error::ErrorCode::SignaturePublicKeyMismatch
        );
        signers.insert(validator);
    }
    require!(
        signers.len() >= threshold as usize,
        crate::error::ErrorCode::SignaturesLess
    );
    msg!("All {} signatures verified successfully!", signatures.len());
    Ok(())
}

fn get_current_chain() -> Chain {
    Chain {
        chain_type: ChainType::Solana as u8,
        chain_id: CHAIN_ID,
    }
}

#[derive(Accounts)]
#[instruction(to_chain: Chain)]
pub struct InitSendToChainNonce<'info> {
    #[account(mut, constraint = sender.key() == bridge_config.admin)]
    pub sender: Signer<'info>,
    #[account(init, payer = sender, seeds = [&to_chain.combain_chain(), ToChainNonce::SEED_SUFFIX.as_bytes()], bump, space = 8 + ToChainNonce::LEN)]
    pub to_chain_nonce_account: Account<'info, ToChainNonce>,
    #[account(seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, constraint = super_admin.key() == bridge_config.admin)]
    pub super_admin: Signer<'info>,
    /// CHECK:
    #[account(mut, seeds = [MESSAGE_FEE_SEED.as_bytes()], bump)]
    pub message_fee: AccountInfo<'info>,
    #[account(seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(to_chain: Chain)]
pub struct SendToOtherChain<'info> {
    #[account(mut, seeds = [&to_chain.combain_chain(), ToChainNonce::SEED_SUFFIX.as_bytes()], bump)]
    pub to_chain_nonce_account: Account<'info, ToChainNonce>,
    #[account(mut)]
    pub sender: Signer<'info>,
    /// CHECK:
    #[account(mut, seeds = [MESSAGE_FEE_SEED.as_bytes()], bump)]
    pub message_fee: AccountInfo<'info>,
    #[account(seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
    #[account(
        seeds = [b"bridge"],
        bump,
        seeds::program = caller_program.key()
    )]
    pub caller_auth_pda: Signer<'info>,
    /// CHECK:
    pub caller_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(msg_header: MsgHeader)]
pub struct ConfirmFromOtherChain<'info> {
    #[account(init_if_needed, payer = user, seeds = [&msg_header.from_chain.combain_chain(), FromChainNonce::SEED_SUFFIX.as_bytes(), receiver.key().as_ref()], bump, space = 8 + FromChainNonce::LEN)]
    pub from_chain_nonce_account: Account<'info, FromChainNonce>,
    #[account(seeds = [ConfigInfo::SEEDS.as_bytes()], bump)]
    pub bridge_config: Account<'info, ConfigInfo>,
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK:
    pub receiver: AccountInfo<'info>,
    #[account(
        seeds = [b"bridge"],
        bump,
        seeds::program = caller_program.key()
    )]
    pub caller_auth_pda: Signer<'info>,
    /// CHECK:
    #[account(mut, constraint = msg_header.to_addr == caller_program.key().to_bytes())]
    pub caller_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
