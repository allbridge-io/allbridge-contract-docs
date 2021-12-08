//! State transition types
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::borsh::try_from_slice_unchecked;
use solana_program::rent::Rent;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};

pub mod asset;
pub mod bridge;

pub use asset::Asset;
pub use bridge::Bridge;

pub type TxId = [u8; 64];
pub type Address = [u8; 32];
pub type BlockchainId = [u8; 4];
pub type LockId = u128;

pub const BLOCKCHAIN_ID: &[u8; 4] = b"SOL\0";
pub const SYSTEM_PRECISION: u8 = 9;

#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum AccountKey {
    Uninitialized,
    Bridge,
    Manager,
    Token, // and LocalToken
}

pub fn assert_uninitialized(account: &AccountInfo) -> ProgramResult {
    let data = &account.data.borrow();
    if data.len() > 0 && data[0] == AccountKey::Uninitialized as u8 {
        Ok(())
    } else {
        Err(ProgramError::AccountAlreadyInitialized)
    }
}

pub fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo, size: usize) -> ProgramResult {
    if account_info.data_len() < size {
        return Err(ProgramError::AccountDataTooSmall);
    }
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        msg!(&rent.minimum_balance(account_info.data_len()).to_string());
        Err(ProgramError::AccountNotRentExempt)
    } else {
        Ok(())
    }
}

pub fn try_from_slice_checked<T: BorshDeserialize>(
    data: &[u8],
    data_type: AccountKey,
    data_size: usize,
) -> Result<T, ProgramError> {
    if data[0] != data_type as u8 || data.len() != data_size {
        Err(ProgramError::InvalidAccountData)
    } else {
        Ok(try_from_slice_unchecked(data)?)
    }
}
