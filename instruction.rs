//! Instruction types

use crate::state::{Address, Asset, BlockchainId, LockId};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program, sysvar,
};


#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct LockArgs {
    /// Recipient address
    pub recipient: Address,
    /// Destination blockchain id
    pub destination: BlockchainId,
    /// Amount
    pub amount: u64,
    /// Lock id
    pub lock_id: LockId,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct UnlockArgs {
    /// Lock id
    pub lock_id: LockId,
    /// Source
    pub lock_source: BlockchainId,
    /// Amount
    pub amount: u64,
    /// Token source
    pub token_source: BlockchainId,
    /// Token source address
    pub token_source_address: Address,
    pub secp_instruction_index: u8,
}

/// Instruction definition
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum BridgeProgramInstruction {
    /// Initializes new bridge account
    InitBridge,
    /// Add token instruction struct
    AddToken,
    /// Remove token instruction struct
    RemoveToken,

    /// Lock (send) tokens
    Lock(LockArgs),

    /// Unlock (receive) tokens
    Unlock(UnlockArgs),

    /// Update bridge owner
}

#[allow(clippy::too_many_arguments)]
pub fn lock(
    program_id: &Pubkey,
    bridge: &Pubkey,
    mint: &Pubkey,
    sender: &Pubkey,
    sender_token_account: &Pubkey,
    bridge_token_account: &Pubkey,
    fee_collector: &Pubkey,
    validator: &Pubkey,
    pool: &Pubkey,
    user_pool_token_account: &Pubkey,
    validator_program_id: &Pubkey,
    recipient: Address,
    destination: String,
    amount: u64,
    lock_id: LockId,
) -> Result<Instruction, ProgramError> {
    let init_data = BridgeProgramInstruction::Lock(LockArgs {
        recipient,
        destination: str_to_chain_id(destination.as_str()),
        amount,
        lock_id,
    });
    let data = init_data
        .try_to_vec()
        .or(Err(ProgramError::InvalidArgument))?;

    let bridge_authority = Pubkey::find_program_address(&[bridge.as_ref()], program_id).0;

    let lock_account = Pubkey::find_program_address(
        &[validator.as_ref(), &lock_id.to_be_bytes(), b"lock"],
        validator_program_id,
    )
    .0;

    let asset_by_mint = seeds_to_pubkey(
        program_id,
        &Asset::get_asset_by_mint_signer_seeds(bridge, mint)?,
    );

    let accounts = vec![
        AccountMeta::new_readonly(*bridge, false),
        AccountMeta::new(bridge_authority, false),
        AccountMeta::new(*mint, false),
        AccountMeta::new_readonly(asset_by_mint, false),
        AccountMeta::new(*sender, true),
        AccountMeta::new(*sender_token_account, false),
        AccountMeta::new(*bridge_token_account, false),
        AccountMeta::new(*fee_collector, false),
        AccountMeta::new(*validator, false),
        AccountMeta::new_readonly(*pool, false),
        AccountMeta::new_readonly(*user_pool_token_account, false),
        AccountMeta::new(lock_account, false),
        AccountMeta::new_readonly(*validator_program_id, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn unlock(
    program_id: &Pubkey,
    bridge: &Pubkey,
    recipient: &Pubkey,
    recipient_token_account: &Pubkey,
    validator: &Pubkey,
    bridge_token_account: &Pubkey,
    mint: &Pubkey,
    payer: &Pubkey,
    fee_collector: &Pubkey,
    validator_program_id: &Pubkey,
    lock_id: LockId,
    source: String,
    amount: u64,
    token_source: String,
    token_source_address: Address,
    secp_instruction_index: u8,
) -> Result<Instruction, ProgramError> {
    let init_data = BridgeProgramInstruction::Unlock(UnlockArgs {
        lock_id,
        lock_source: str_to_chain_id(source.as_str()),
        amount,
        token_source: str_to_chain_id(token_source.as_str()),
        token_source_address,
        secp_instruction_index,
    });
    let data = init_data
        .try_to_vec()
        .or(Err(ProgramError::InvalidArgument))?;

    let bridge_authority = Pubkey::find_program_address(&[bridge.as_ref()], program_id).0;

    let seed = format!("unlock_{}", source);
    let unlock_account = Pubkey::find_program_address(
        &[validator.as_ref(), &lock_id.to_be_bytes(), seed.as_bytes()],
        validator_program_id,
    )
    .0;

    let asset_by_source = seeds_to_pubkey(
        program_id,
        &Asset::get_asset_by_source_signer_seeds(
            bridge,
            str_to_chain_id(token_source.as_str()),
            token_source_address,
        )?,
    );

    let accounts = vec![
        AccountMeta::new_readonly(*bridge, false),
        AccountMeta::new_readonly(bridge_authority, false),
        AccountMeta::new(*recipient, false),
        AccountMeta::new_readonly(*validator, false),
        AccountMeta::new(*recipient_token_account, false),
        AccountMeta::new_readonly(asset_by_source, false),
        AccountMeta::new(*bridge_token_account, false),
        AccountMeta::new(*mint, false),
        AccountMeta::new(unlock_account, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new(*fee_collector, false),
        AccountMeta::new_readonly(*validator_program_id, false),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

pub fn seeds_to_pubkey(program_id: &Pubkey, seed: &[Vec<u8>]) -> Pubkey {
    let vec_of_slice = seed.iter().map(|v| v.as_slice()).collect::<Vec<&[u8]>>();
    let seeds = vec_of_slice.as_slice();
    Pubkey::find_program_address(seeds, program_id).0
}

pub fn str_to_chain_id(str: &str) -> [u8; 4] {
    let str_len = str.len();
    let mut result = [0; 4];
    result[..str_len].copy_from_slice(str.as_bytes());
    result
}