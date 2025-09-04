use crate::{state::config::MsgHeader, Chain};
use anchor_lang::prelude::*;

#[event]
pub struct SendMessage {
    pub header: MsgHeader,
    pub body: Vec<u8>,
    pub fee: u64,
}

#[event]
pub struct ConfirmMessage {
    pub executor: Pubkey,
    pub from_chain: Chain,
    pub nonce: u64,
    pub mbody: Vec<u8>,
}
