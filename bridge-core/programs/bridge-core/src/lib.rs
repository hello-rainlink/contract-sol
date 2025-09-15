use anchor_lang::prelude::*;
use instructions::{
    config::{self, *},
    message::{self, *},
    validator::{self},
};
use state::config::*;

mod error;
mod evnet;
pub mod instructions;
pub mod state;
mod utils;

declare_id!("E9y3Whtskj2Jt4JBQZTenn5pvVthupucpSXfozhQFhQW");

#[program]
pub mod bridge_core {
    use super::*;

    pub fn initialize(ctx: Context<ConfInitialize>, bump: u8) -> Result<()> {
        config::initialize(ctx, bump)?;
        Ok(())
    }

    pub fn change_admin(ctx: Context<BridgeConf>, new_admin: Pubkey) -> Result<()> {
        config::change_admin(ctx, new_admin)?;
        Ok(())
    }

    pub fn set_threshold(ctx: Context<BridgeConf>, new_threshold: u8) -> Result<()> {
        validator::set_threshold(ctx, new_threshold)?;
        Ok(())
    }

    pub fn add_signers(ctx: Context<BridgeConf>, new_signers: Vec<[u8; 20]>) -> Result<()> {
        validator::add_signers(ctx, new_signers)?;
        Ok(())
    }

    pub fn remove_signers(
        ctx: Context<BridgeConf>,
        signers_to_remove: Vec<[u8; 20]>,
    ) -> Result<()> {
        validator::remove_signers(ctx, signers_to_remove)?;
        Ok(())
    }

    pub fn set_bridge_fee(ctx: Context<BridgeConf>, bridge_fee: u64) -> Result<()> {
        message::set_bridge_fee(ctx, bridge_fee)?;
        Ok(())
    }

    pub fn init_to_chain_nonce_account(
        ctx: Context<InitSendToChainNonce>,
        to_chain: Chain,
    ) -> Result<()> {
        message::init_to_chain_nonce_account(ctx, to_chain)?;
        Ok(())
    }

    pub fn send_message(
        ctx: Context<SendToOtherChain>,
        to_chain: Chain,
        to_addr: [u8; 32],
        mbody: Vec<u8>,
        mtype: u8,
        upload_fee: u64,
    ) -> Result<()> {
        message::send_message(ctx, to_chain, to_addr, mbody, mtype, upload_fee)?;
        Ok(())
    }

    pub fn confirm_message(
        ctx: Context<ConfirmFromOtherChain>,
        msg_header: MsgHeader,
        msg_body: Vec<u8>,
        accum_pk: Vec<u8>,
        signatures: Vec<[u8; 65]>,
    ) -> Result<()> {
        message::confirm_message(ctx, msg_header, msg_body, accum_pk, signatures)?;
        Ok(())
    }

    pub fn withdraw_fee(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        message::withdraw_fee(ctx, amount)?;
        Ok(())
    }
}
