use anchor_lang::prelude::*;

// 2^32
pub const SCALING_FACTOR: u64 = 1u64 << 32;
// 365*24*60*60 = 31536000
const SECONDS_PER_YEAR_SCALED: u64 = 31536000 * SCALING_FACTOR;

#[account]
pub struct Lp {
    pub amount: u64,
    pub earns: u64,
    pub debt: u64,
    pub remaining: u64,
    pub padding: [u8; 32],
}
impl Lp {
    pub const LEN: usize = 8 * 4 + 32;
    pub const SEEDS: &str = super::LP_SEED;
}

#[account]
pub struct Pool {
    pub token_mint: Pubkey,
    pub total_liquidity: u64,
    pub total_earns: u64,
    pub acc_ratio: u64,
    pub last_apy: u64,
    pub last_receive_rewards_time: i64,
    pub platform_vault: i64,
    pub total_staked: u64,
    pub total_staked_liquidity: u64,
    pub pool_fee_rate: u64,
    pub padding: [u8; 8],
}
impl Pool {
    pub const LEN: usize = 32 + 8 * 8 + 32;
    pub const SEEDS: &str = super::POOL_SEED;
    #[inline(never)]
    pub fn transfer_to_pool(&mut self, amount: i64) -> Result<()> {
        self.total_liquidity += amount as u64;
        self.platform_vault += amount;
        Ok(())
    }
    #[inline(never)]
    pub fn transfer_from_pool(&mut self, amount: i64) -> Result<()> {
        self.total_liquidity -= amount as u64;
        self.platform_vault -= amount;
        Ok(())
    }
    #[inline(never)]
    pub fn refresh_rewards(&mut self, balance: u64, lp_fee: u64, all_amount: u64) -> Result<()> {
        self.total_liquidity = balance;

        let pool_fee = ((lp_fee as u128) * (self.total_staked as u128)
            / (self.total_liquidity as u128)) as u64;

        // Calculate the decrease in the staked amount.
        let staked_decrease = ((all_amount as u128) * (self.total_staked as u128)
            / (self.total_liquidity as u128)) as u64;
        self.total_staked -= staked_decrease;

        // Return the remaining pool fee
        self.total_liquidity = self.total_liquidity - all_amount + lp_fee - pool_fee;
        let vault_delta = all_amount - lp_fee + pool_fee;
        self.platform_vault -= vault_delta as i64;

        let now_seconds = Clock::get()?.unix_timestamp;
        let delta = now_seconds - self.last_receive_rewards_time;
        self.total_earns += pool_fee;

        let scaled_fee = ((pool_fee as u128) * (SCALING_FACTOR as u128)) as u64;
        let ratio_increment = (scaled_fee as u128 / self.total_staked_liquidity as u128) as u64;
        self.acc_ratio += ratio_increment;

        let numerator = ((pool_fee as u128) * (SECONDS_PER_YEAR_SCALED as u128)) as u64;
        let denominator = ((self.total_staked_liquidity as u128) * (delta.abs() as u128)) as u64;
        self.last_apy = if denominator == 0 {
            self.last_apy
        } else {
            (numerator as u128 / denominator as u128) as u64
        };

        self.last_receive_rewards_time = now_seconds;
        Ok(())
    }
}
