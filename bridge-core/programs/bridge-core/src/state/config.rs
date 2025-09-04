use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use std::{collections::BTreeMap, str};

pub const CONFIG_SEED: &str = "global";
pub const TO_NONCE_SEED: &str = "toNonce";
pub const FROM_NONCE_SEED: &str = "fromNonce";
pub const MESSAGE_FEE_SEED: &str = "vaultFee";

#[cfg(feature = "mainnet")]
pub const CHAIN_ID: u64 = 101;
#[cfg(feature = "testnet")]
pub const CHAIN_ID: u64 = 102;
#[cfg(feature = "devnet")]
pub const CHAIN_ID: u64 = 103;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum ChainType {
    Ethereum,
    TRON,
    Solana,
}

#[derive(Debug, Clone, AnchorDeserialize, AnchorSerialize, PartialEq)]
pub struct Chain {
    pub chain_type: u8,
    pub chain_id: u64,
}
impl Chain {
    pub const LEN: usize = 1 + 8;

    pub fn combain_chain(&self) -> Vec<u8> {
        let mut combined_bytes = Vec::new();
        combined_bytes.extend_from_slice(&self.chain_type.to_be_bytes());
        combined_bytes.extend_from_slice(&self.chain_id.to_be_bytes());
        combined_bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let chain_type = u8::from_be_bytes(bytes[0..1].try_into().unwrap());
        let chain_id = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
        Self {
            chain_type,
            chain_id,
        }
    }
}

#[derive(Clone, Debug, AnchorDeserialize, AnchorSerialize, PartialEq)]
pub struct MsgHeader {
    pub mtype: u8,
    pub nonce: u64,
    pub from_chain: Chain,
    pub from_addr: [u8; 32],
    pub to_chain: Chain,
    pub to_addr: [u8; 32],
    pub upload_gas_fee: u128,
}
impl MsgHeader {
    pub fn to_be_bytes(&self) -> Vec<u8> {
        [
            self.mtype.to_be_bytes().to_vec(),
            self.nonce.to_be_bytes().to_vec(),
            self.from_chain.combain_chain().to_vec(),
            self.from_addr.to_vec(),
            self.to_chain.combain_chain().to_vec(),
            self.to_addr.to_vec(),
            self.upload_gas_fee.to_be_bytes().to_vec(),
        ]
        .concat()
    }
}

#[account]
#[derive(Debug)]
pub struct ConfigInfo {
    pub admin: Pubkey,
    pub signers: Vec<[u8; 20]>,
    pub threshold: u8,
    pub bridge_fee: u64,
    pub bump: u8,
    pub padding: [u8; 136],
}
impl ConfigInfo {
    pub const LEN: usize = 32 + 4 + 20 * 12 + 1 + 8 + 1 + 136;
    pub const SEEDS: &str = CONFIG_SEED;
}

#[account]
pub struct ToChainNonce {
    pub chain: Chain,
    pub max_nonce: u64,
}
impl ToChainNonce {
    pub const LEN: usize = 9 + 8;
    pub const SEED_SUFFIX: &str = TO_NONCE_SEED;
}

#[account]
pub struct FromChainNonce {
    pub chain: Chain,
    pub last_nonce: u64,
    pub missing_nonces: Vec<u8>,
}
impl FromChainNonce {
    pub const LEN: usize = 9 + 8 + 4 + 50 * 16;
    pub const SEED_SUFFIX: &str = FROM_NONCE_SEED;

    // Convert Vec<u8> to BTreeMap<u64, ()>
    fn vec_to_btreemap(&self) -> Result<BTreeMap<u64, ()>> {
        if self.missing_nonces.is_empty() {
            msg!("missing_nonces is empty");
            return Ok(BTreeMap::new());
        }

        match bincode::deserialize::<BTreeMap<u64, ()>>(&self.missing_nonces) {
            Ok(map) => return Ok(map),
            Err(e) => {
                msg!("Error deserializing missing_nonces: {:?}", e);
                msg!("missing_nonces content: {:?}", self.missing_nonces);
                return Err(crate::error::ErrorCode::DeserializationError.into());
            }
        }
    }

    // Convert BTreeMap<u64, ()> to Vec<u8>
    fn btreemap_to_vec(map: &BTreeMap<u64, ()>) -> Result<Vec<u8>> {
        let data = bincode::serialize(map).unwrap();
        Ok(data)
    }

    pub fn check_and_store_nonce(&mut self, nonce: u64) -> Result<bool> {
        let mut map = self.vec_to_btreemap()?;
    
        if map.is_empty() {
            map.insert(nonce, ());
            self.missing_nonces = Self::btreemap_to_vec(&map)?;
            return Ok(true);
        }
    
        if map.contains_key(&nonce) {
            return Err(crate::error::ErrorCode::NonceConsumed.into());
        }
    
        let min_nonce = *map.keys().next().ok_or(crate::error::ErrorCode::NonceInvalid)?;
    
        if nonce < min_nonce {
            return Err(crate::error::ErrorCode::NonceInvalid.into());
        }
    
        if map.len() >= 50 {
            map.remove(&min_nonce);
        }
    
        map.insert(nonce, ());
        self.missing_nonces = Self::btreemap_to_vec(&map)?;
        Ok(true)
    }
}
