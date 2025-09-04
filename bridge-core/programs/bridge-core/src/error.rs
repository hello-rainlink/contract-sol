use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Validator count cannot exceed 12")]
    ValidatorOver12,
    #[msg("Number of signatures is less than threshold")]
    SignaturesLess,
    #[msg("Invalid threshold")]
    InvalidThreshold,
    #[msg("Number of signatures does not match number of public keys")]
    SignaturePublicKeyMismatch,
    #[msg("Signature verification failed")]
    SignatureVerificationFailed,
    #[msg("Nonce invalid")]
    NonceInvalid,
    #[msg("Nonce is consumed")]
    NonceConsumed,
    #[msg("Insufficient fee")]
    InsufficientFee,
    #[msg("Failed to deserialize missing_nonces")]
    DeserializationError,
    #[msg("Receiver does not match")]
    ReceiverMismatch,
}