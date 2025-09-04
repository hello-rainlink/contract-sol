use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Total liquidity not enough")]
    TotalIlliquidity,
    #[msg("User liquidity not enough")]
    Illiquidity,
    #[msg("Withdrawal amount exceed reward")]
    WithdrawalAmountExceedReward,
    #[msg("Chain combain not match")]
    ChainCombainNotMatch,
    #[msg("Conversion error")]
    ConversionError,
    #[msg("Sender address not match")]
    SenderAddrNotMatch,
    #[msg("Execute address not match")]
    ExecuteAddrNotMatch,
    #[msg("Mint type not supported")]
    MintTypeNotSupported,
    #[msg("Token relation not found")]
    TokenRelationNotFound,
    #[msg("Staked decrease too large")]
    StakedDecreaseTooLarge,
}
