use super::{try_from_slice_checked, AccountKey};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Information about the bridge
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Bridge {
    /// Account type
    pub key: AccountKey,
    /// Bridge owner account, signs secure instructions to the bridge
    pub owner: Pubkey,
    pub token_manager: Pubkey,
    pub active: bool,
    pub validator_program_id: Pubkey,
    pub validator: Pubkey,
    pub authority_bump_seed: u8,
    pub unlock_signer: Pubkey,
    pub base_fee_rate_bp: u64,
    pub pool: Pubkey,
    pub fee_multiplier: u64,
}

const BP: u128 = 10000;
const BP_SQUARED: u128 = BP * BP;

impl Bridge {
    /// Struct size
    pub const LEN: usize = 1 + 32 + 32 + 1 + 32 + 32 + 1 + 32 + 8 + 32 + 8;
    /// Create new bridge entity
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner: Pubkey,
        validator_program_id: Pubkey,
        validator: Pubkey,
        authority_bump_seed: u8,
        unlock_signer: Pubkey,
        base_fee_rate_bp: u64,
        pool: Pubkey,
        fee_multiplier: u64,
    ) -> Self {
        Self {
            key: AccountKey::Bridge,
            owner,
            token_manager: owner,
            active: true,
            validator_program_id,
            validator,
            authority_bump_seed,
            unlock_signer,
            base_fee_rate_bp,
            pool,
            fee_multiplier,
        }
    }

    pub fn from_account_info(account: &AccountInfo) -> Result<Self, ProgramError> {
        try_from_slice_checked(&account.data.borrow_mut(), AccountKey::Bridge, Self::LEN)
    }

    pub fn assert_validator(
        &self,
        validator_program_id: &Pubkey,
        validator: &Pubkey,
    ) -> ProgramResult {
        if self.validator_program_id != *validator_program_id || self.validator != *validator {
            msg!("Invalid validator data");
            return Err(ProgramError::InvalidArgument);
        }
        Ok(())
    }

    pub fn calculate_fee(
        &self,
        transfer_amount: u64,
        stake_size: u64,
        pool_size: u64,
    ) -> Result<u64, ProgramError> {
        msg!(
            "Calculate fee: {:?}, {:?}, {:?}, {:?}. {:?}",
            transfer_amount,
            stake_size,
            pool_size,
            self.base_fee_rate_bp,
            self.fee_multiplier
        );
        if pool_size == 0 || transfer_amount == 0 || self.base_fee_rate_bp == 0 {
            return Ok(0);
        }
        // fee_multiplier * self.amount * BP / pool_size
        let user_share_bp =
            (self.fee_multiplier as u128).checked_mul_overflow(stake_size as u128)?;
        let user_share_bp = user_share_bp.checked_mul_overflow(BP)?;
        let user_share_bp = user_share_bp / (pool_size as u128); // pool_size checked to be non-0 above

        // (BP * BP / base_fee_rate_bp)
        let base_fee_adj_bp = BP_SQUARED / (self.base_fee_rate_bp as u128);

        // (amount * BP) / (user_share_bp + base_fee_adj)
        let user_share_plus_adj_bp = user_share_bp.checked_add_overflow(base_fee_adj_bp)?;
        let amount_bp = (transfer_amount as u128).checked_mul_overflow(BP)?;
        let result = amount_bp / user_share_plus_adj_bp;
        Ok(result as u64)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use solana_program::native_token::sol_to_lamports;

    #[test]
    fn test_calculate_fee() {
        let amount: u64 = sol_to_lamports(1.0);
        let base_fee: u64 = 30;
        let fee_multiplier: u64 = 200000;
        let pool_size: u64 = sol_to_lamports(100000.0);

        let bridge = Bridge::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            base_fee,
            Pubkey::new_unique(),
            fee_multiplier,
        );

        let stake_size = sol_to_lamports(0.0);
        assert_eq!(
            bridge.calculate_fee(amount, stake_size, pool_size).unwrap(),
            3000000
        );

        let stake_size = sol_to_lamports(10.0);
        assert_eq!(
            bridge.calculate_fee(amount, stake_size, pool_size).unwrap(),
            2830188
        );

        let stake_size = sol_to_lamports(30.0);
        assert_eq!(
            bridge.calculate_fee(amount, stake_size, pool_size).unwrap(),
            2542373
        );

        let stake_size = sol_to_lamports(100.0);
        assert_eq!(
            bridge.calculate_fee(amount, stake_size, pool_size).unwrap(),
            1875000
        );

        let stake_size = sol_to_lamports(300.0);
        assert_eq!(
            bridge.calculate_fee(amount, stake_size, pool_size).unwrap(),
            1071428
        );

        let stake_size = sol_to_lamports(1000.0);
        assert_eq!(
            bridge.calculate_fee(amount, stake_size, pool_size).unwrap(),
            428571
        );

        let stake_size = sol_to_lamports(10000.0);
        assert_eq!(
            bridge.calculate_fee(amount, stake_size, pool_size).unwrap(),
            49180
        );

        let stake_size = sol_to_lamports(100000.0);
        assert_eq!(
            bridge.calculate_fee(amount, stake_size, pool_size).unwrap(),
            4991
        );
    }
}
