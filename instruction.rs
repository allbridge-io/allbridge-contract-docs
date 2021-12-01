//! Instruction types

use crate::state::{Address, BlockchainId};
use crate::utils::*;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program, sysvar,
};

/// Instruction definition
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum BridgeProgramInstruction {
    InitializeBridge,
    AddToken,
    RemoveToken,
    Lock {
        /// Recipient address
        recipient: Address,
        /// Destination blockchain id
        destination: BlockchainId,
        /// Amount
        amount: u64,
    },

    /// Lock instruction data
    Claim {
        /// Lock id
        lock_id: u64,
        /// Source
        source: BlockchainId,
        /// Amount
        amount: u64,
        /// Token source
        token_source: BlockchainId,
        /// Token source address
        token_source_address: Address,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn lock(
    program_id: &Pubkey,
    /// Recipient address
    recipient: Address,
    /// Destination
    destination: String,
    /// Amount
    amount: u64,
    /// Bridge account
    bridge_account: &Pubkey,
    /// Transferred token mint account
    mint_account: &Pubkey,
    /// Account with information about token (Token struct) derived by [bridge_account, mint_account, "token"]
    local_token_account: &Pubkey,
    /// Sender main account, should be signed
    sender_account: &Pubkey,
    /// Sender token account
    sender_token_account: &Pubkey,
    /// Bridge token account, derived by [bridge_account, mint_account, "spl_token"]
    bridge_token_account: &Pubkey,
    /// Fee collector account - is in local_token_account (Token struct)
    fee_collector_account: &Pubkey,
    /// Account to create lock info (Lock struct), derived by [bridge_account, format!("lock_{}", lock_id)]
    lock_account: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = BridgeProgramInstruction::Lock {
        recipient,
        destination: str_to_chain_id(destination.as_str()),
        amount,
    };
    let data = init_data
        .try_to_vec()
        .or(Err(ProgramError::InvalidArgument))?;
    let accounts = vec![
        AccountMeta::new(*bridge_account, false),
        AccountMeta::new(*mint_account, false),
        AccountMeta::new_readonly(*local_token_account, false),
        AccountMeta::new(*sender_account, true),
        AccountMeta::new(*sender_token_account, false),
        AccountMeta::new(*bridge_token_account, false),
        AccountMeta::new(*fee_collector_account, false),
        AccountMeta::new(*lock_account, false),
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
pub fn claim(
    program_id: &Pubkey,
    lock_id: u64,
    source: String,
    amount: u64,
    token_source: String,
    token_source_address: Address,
    bridge_account: &Pubkey,
    recipient_account: &Pubkey,
    recipient_token_account: &Pubkey,
    token_account: &Pubkey,
    bridge_token_account: &Pubkey,
    mint_account: &Pubkey,
    claimed_account: &Pubkey,
    payer_account: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = BridgeProgramInstruction::Claim {
        lock_id,
        source: str_to_chain_id(source.as_str()),
        amount,
        token_source: str_to_chain_id(token_source.as_str()),
        token_source_address,
    };
    let data = init_data
        .try_to_vec()
        .or(Err(ProgramError::InvalidArgument))?;

    let (bridge_authority, _) =
        Pubkey::find_program_address(&[bridge_account.as_ref()], program_id);

    let accounts = vec![
        AccountMeta::new_readonly(*bridge_account, false),
        AccountMeta::new_readonly(bridge_authority, false),
        AccountMeta::new(*recipient_account, false),
        AccountMeta::new(*recipient_token_account, false),
        AccountMeta::new_readonly(*token_account, false),
        AccountMeta::new(*bridge_token_account, false),
        AccountMeta::new(*mint_account, false),
        AccountMeta::new(*claimed_account, false),
        AccountMeta::new(*payer_account, true),
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
