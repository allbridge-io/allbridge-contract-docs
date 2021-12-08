use super::{try_from_slice_checked, AccountKey};
use super::{Address, BlockchainId};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Token info
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Asset {
    /// Account type
    pub key: AccountKey,
    /// Bridge reference
    pub bridge: Pubkey,
    /// Token source blockchain id
    pub source: BlockchainId,
    /// Source address
    pub source_address: Address,
    /// Token precision
    pub decimals: u8,
    /// Token symbol
    pub symbol: [u8; 12],
    /// Token name
    pub name: [u8; 32],
    /// Token mint account
    pub mint: Pubkey,
    /// Bridge token account
    pub token_account: Pubkey,
    /// Minimal token fee
    pub min_fee: u64,
    /// Account to collect fee
    pub fee_collector: Pubkey,
    /// If this token is mapped to local token even if it has another source
    pub is_wrapped: bool,

    pub enabled: bool,
}

impl Asset {
    /// Struct size
    pub const LEN: usize = 1 + 32 + 4 + 32 + 1 + 12 + 32 + 32 + 32 + 8 + 32 + 1 + 1;

    pub fn from_account_info(account: &AccountInfo) -> Result<Self, ProgramError> {
        try_from_slice_checked(&account.data.borrow_mut(), AccountKey::Token, Self::LEN)
    }

    pub fn assert_bridge_account(&self, bridge_account: &Pubkey) -> ProgramResult {
        if self.bridge != *bridge_account {
            msg!("Token info account is from another bridge");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    pub fn assert_token_account(&self, token_account: &Pubkey) -> ProgramResult {
        if self.token_account != *token_account {
            msg!("Invalid token account");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    pub fn get_asset_by_source_signer_seeds(
        bridge: &Pubkey,
        source: BlockchainId,
        source_address: Address,
    ) -> Result<Vec<Vec<u8>>, ProgramError> {
        let seed = format!("asset_{}", chain_id_to_str(&source)?);
        Ok(vec![
            bridge.as_ref().to_vec(),
            source_address.as_ref().to_vec(),
            seed.as_bytes().to_vec(),
        ])
    }

    pub fn get_asset_by_mint_signer_seeds(
        bridge: &Pubkey,
        mint: &Pubkey,
    ) -> Result<Vec<Vec<u8>>, ProgramError> {
        Ok(vec![
            bridge.as_ref().to_vec(),
            mint.as_ref().to_vec(),
            b"asset".to_vec(),
        ])
    }
}
