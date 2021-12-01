pub type Address = [u8; 32];
pub type BlockchainId = [u8; 4];

#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum AccountKey {
    Uninitialized,
    Bridge,
    Token, // and LocalToken
    Lock,
    Claimed,
}

#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Bridge {
    /// Account type
    pub key: AccountKey,
    /// Bridge owner account, signs secure instructions to the bridge
    pub owner: Pubkey,
    /// Number of locks
    pub locks: u64,
    /// Oracle address
    pub oracle_address: [u8; 20],
}

#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Claimed {
    /// Account type
    pub key: AccountKey,
    /// Bridge account
    pub bridge: Pubkey,
    /// Source blockchain identifier
    pub source: BlockchainId,
    /// Claimed transfer ID
    pub lock_id: u64,
}

#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Lock {
    /// Account type
    pub key: AccountKey,
    /// Bridge reference
    pub bridge: Pubkey,
    /// Lock index within the bridge
    pub index: u64,
    /// Recipient address
    pub recipient: Address,
    /// Destination blockchain identifier
    pub destination: BlockchainId,
    /// Amount to lock for the transfer
    pub amount: u64,
    /// Token source blockchain id
    pub token_source: BlockchainId,
    /// Token source address
    pub source_address: Address,
}

#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Token {
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
    /// Fee for the token
    pub fee: u64,
    /// Account to collect fee
    pub fee_collector: Pubkey,
    /// If this token is mapped to local token even if it has another source
    pub is_wrapped: bool,
}
