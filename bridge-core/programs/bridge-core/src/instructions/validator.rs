use std::cmp::min;

use crate::error::ErrorCode;
use anchor_lang::prelude::*;

use super::config::BridgeConf;

pub fn add_signers(ctx: Context<BridgeConf>, new_signers: Vec<[u8; 20]>) -> Result<()> {
    let bridge_config = &mut ctx.accounts.bridge_config;
    for signer in new_signers {
        if !bridge_config.signers.contains(&signer) {
            bridge_config.signers.push(signer);
        }
    }
    require!(
        bridge_config.signers.len() <= 12,
        crate::error::ErrorCode::ValidatorOver12
    );
    Ok(())
}

pub fn remove_signers(ctx: Context<BridgeConf>, signers_to_remove: Vec<[u8; 20]>) -> Result<()> {
    let bridge_config = &mut ctx.accounts.bridge_config;
    bridge_config
        .signers
        .retain(|signer| !signers_to_remove.contains(signer));

    if bridge_config.signers.len() == 0 {
        return Err(ErrorCode::InvalidThreshold.into());
    }

    bridge_config.threshold = min(bridge_config.threshold, bridge_config.signers.len() as u8);

    Ok(())
}

/// Sets the threshold for the multisig account.
pub fn set_threshold(ctx: Context<BridgeConf>, new_threshold: u8) -> Result<()> {
    let bridge_config = &mut ctx.accounts.bridge_config;

    if new_threshold == 0 || new_threshold as usize > bridge_config.signers.len() {
        return Err(ErrorCode::InvalidThreshold.into());
    }

    bridge_config.threshold = new_threshold;
    Ok(())
}
