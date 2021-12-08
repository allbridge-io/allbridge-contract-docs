# Allbridge API

## Intro

This document explains how to use Allbridge to transfer assets programmatically as well as integrating Allbridge into your own UI.

Generally speaking, you need three steps to transfer assets using Allbridge:

1. Invoke Allbridge smart contract on Blockchain #1 to lock funds
2. Use Allbridge API to get confirmation signature about funds being locked on Blockchain #1
3. Invoke Allbridge smart contract on Blockchain #2 using the signature received in the previous step to unlock the funds

There are also some helper methods to list supported tokens and their fees, check recipient address validity etc. All are listed in the sections below.

## Contents

- [Main transfer flow](#main-transfer-flow)
  - [Lock tokens](#lock-tokens)
  - [Get signature](#get-signature)
  - [Unlock tokens](#unlock-tokens)
- [Utility endpoints](#utility-endpoints)
  - [List supported tokens](#list-supported-tokens)
  - [Check recipient address](#check-recipient-address)
  - [Check recipient token balance](#check-recipient-token-balance)
- [Fee calculation](#fee-calculation)
- [Constants](#constants)
  - [Blockchain IDs](#blockchain-ids)

## Main transfer flow

### Lock tokens

#### EVM

Call `lock` method:

```solidity
    function lock(uint128 lockId, address tokenAddress, bytes32 recipient, bytes4 destination, uint256 amount)
```

- `lockId` a random 16-byte value identifying the transfer within the bridge, with first byte is used as a bridge version (must be `0x01`)
- `tokenAddress` is an address of the token you're trying to lock
- `recipient` recipient address as 32 bytes (zeros at the end if receiving blockchain has addresses shorter than 32 bytes)
- `destination` [Blockchain ID](#blockchain-ids) as 4 bytes (UTF8, zeros at the end if shorter than 4 bytes)
- `amount` amount of lock on EVM side

For native tokens use another method:
```solidity
    function lockBase(uint128 lockId, address wrappedBaseTokenAddress, bytes32 recipient, bytes4 destination) payable
```

- `lockId` a random 16-byte value identifying the transfer within the bridge, with first byte is used as a bridge version (must be `0x01`)
- `wrappedBaseTokenAddress` Wrapped token address (`WETH` address)
- `recipient` recipient address as 32 bytes (zeros at the end if receiving blockchain has addresses shorter than 32 bytes)
- `destination` [Blockchain ID](#blockchain-ids) as 4 bytes (UTF8, zeros at the end)

#### Solana

Use `lock` instruction (see the reference Rust implementation in [instruction.rs](./instruction.rs) file):

##### Instruction data

```rust
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
```

- `recipient` is 32-byte recipient address. For chains with smaller addresses (like EVM) pad address with zeroes at the end until the size is 32 bytes.
- `destination` 4-byte destination [blockchain ID](#blockchain-ids)
- `amount` amount (including fee) to lock in the Solana side
- `lock_id` a random 16-byte value identifying the transfer within the bridge, with first byte is used as a bridge version (must be `0x01`)

##### Accounts

1. Bridge account, use `bb1XfNoER5QC3rhVDaVz3AJp9oFKoHNHG6PHfZLcCjj`
2. Bridge authority, PDA calculated using bridge account (account #1). Or just use `CYEFQXzQM6E5P8ZrXgS7XMSwU3CiqHMMyACX4zuaA2Z4`
3. Mint account of the token you want to send
4. PDA account storing information about the asset. Calculate using bridge (account #1), mint (account #3) and `asset` constant as seeds
5. (**Signer**) Sender account, authority that can be used to transfer tokens to the bridge
6. Sender token account, typically an associated token account for the mint (account #3) and the owner (account #5)
7. Bridge token account to transfer tokens to. It is stored in asset PDA account (account #4), 32 bytes, offset 146 bytes from the start (see `Asset` structure in [state/asset.rs](state/asset.rs)). This account is used for sending native tokens from Solana. For wrapped tokens (which are burned on lock) use any other account here (for example, system account `11111111111111111111111111111111`)
8. Fee collector token account receiving the fee for the transfer. It is stored in asset PDA account (account #4), 32 bytes, offset 186 bytes from the start (see `Asset` structure in [state/asset.rs](state/asset.rs))
9. Validator account, you can read it from bridge account data (account #1). Or just use `7DbBk8bTaw2gxgjjAQHAd4ZKaCYPbhFX63WjEtE5QD6G`
10. Account for the ABR staking pool used to calculate fee, you can read it from bridge account data (account #1). Or just use `s4xknfXUzxLCXUSNgz99tCiPNPfY1bsdvzLVgQfJzd8`
11. User account storing xABR token balance used for balance calculation. Must be an associate token account of the transaction sigher (account #5). If user does not have xABR still specify this account even if it does not exist
12. New account to be created by the bridge to store information about the lock on-chain. PDA calculated for the Validator program using validator (account #9), lock ID (value from the instruction data, see above) and `lock` constant as seeds
13. Validator Program ID, use `va1udM9Gg22vcEbcuu4bsAu6tipRR8KSCHTGbaAhYQk`
14. System rent account, use `SysvarRent111111111111111111111111111111111`
15. SPL Token Program ID, use `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`
16. System Program ID, use `11111111111111111111111111111111`

### Get signature

Call server method with lock transaction id to get info and signature
```http request
GET https://allbridgeapi.net/sign/{transactionId}
```

Response example

```json5
{
  "lockId": "1999368962333213694265338977688250756", // Inner lock id
  "block": "28598359", // Lock transaction block
  "source": "POL", // Transfer source blockchain ID
  "amount": "5000000000", // Amount to receive in system precision (9) (send_amount - bridge_fee)
  "destination": "SOL", // Transfer destination blockchain ID
  "recipient": "0x79726da52d99d60b07ead73b2f6f0bf6083cc85c77a94e34d691d78f8bcafec9", // Recipient address (32 bytes hex, zeros at the end)
  "tokenSource": "SOL", // Token source blockchain ID
  "tokenSourceAddress": "0x069b8857feab8184fb687f634618c035dac439dc1aeb3b5598a0f00000000001", // Token source address
  "signature": "012000000c0" // Signature to pass it to unlock method
}
```

### Unlock tokens

#### EVM

All parameters for unlock is returned by the Allbridge API `sign` method (previous step)

```solidity
function unlock(uint128 lockId, address recipient, uint256 amount, bytes4 lockSource, bytes4 tokenSource, bytes32 tokenSourceAddress, bytes calldata signature)
```

- `lockId` Lock ID value of the initial lock on Blockchain #1 (returned by the `sign` Allbridge API call)
- `recipient` Recipient address returned by the `sign` API call and formatted as EVM address (first 20 bytes)
- `amount` Amount in bridge internal precision (9 digits), use the same amount as returned by the `sign` Allbridge API call
- `lockSource` Transfer source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end)
- `tokenSource` Token source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end)
- `tokenSourceAddress` Token source address, use the same value as returned by the `sign` method
- `signature` Signature for unlock, pass the value received from the `sign` method call

#### Solana

Use `unlock` instruction (see the reference Rust implementation in [instruction.rs](./instruction.rs) file):

##### Instruction data

```rust
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
```

- `lock_id` Lock ID value of the initial lock on Blockchain #1 (returned by the `sign` Allbridge API call)
- `lock_source` Transfer source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end)
- `amount` Amount in bridge internal precision (9 digits), use the same amount as returned by the `sign` Allbridge API call
- `token_source` Token source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end)
- `token_source_address` Token source address, use the same value as returned by the `sign` method
- `secp_instruction_index` Index of the signature verification instruction, typically `0` is used unless you need some instructions to be prior to it in the transaction

##### Accounts

1. Bridge account, use `bb1XfNoER5QC3rhVDaVz3AJp9oFKoHNHG6PHfZLcCjj`
2. Bridge authority, PDA calculated using bridge account (account #1). Or just use `CYEFQXzQM6E5P8ZrXgS7XMSwU3CiqHMMyACX4zuaA2Z4`
3. Recipient account as returned by the `sign` Allbridge API call
4. Validator account, you can read it from bridge account data (account #1). Or just use `7DbBk8bTaw2gxgjjAQHAd4ZKaCYPbhFX63WjEtE5QD6G`
5. Recipient token account, must be an associate token account of the recipient (account #3)
6. PDA account storing information about the asset. Calculate using bridge (account #1), token address on the source chain (32 bytes) and `asset_<CHAIN_ID>` (for example, `asset_ETH`) value as seeds
7. Bridge token account to transfer tokens from. It is stored in asset PDA account (account #6), 32 bytes, offset 146 bytes from the start (see `Asset` structure in [state/asset.rs](state/asset.rs)). This account used for sending native tokens from Solana. For wrapped tokens (which are burned on lock) use any other account here (for example, system account `11111111111111111111111111111111`)
8. Mint account of the token you want to receive
9. New account to be created by the bridge to store information about the unlock on-chain. PDA calculated for the Validator program using validator (account #9), lock ID (value from the instruction data, see above) and `unlock_<CHAIN_ID>` (for example, `unlock_ETH` for tokens sent from Ethereum) value as seeds
10. Account paying for the transaction and unlock account creation. No special authority is required for this transaction, so anyone willing to pay can create one
11. Fee collector token account receiving the fee for the transfer. It is stored in asset PDA account (account #4), 32 bytes, offset 186 bytes from the start (see `Asset` structure in [state/asset.rs](state/asset.rs)). Normally this account is not used because there is no fee on the receiving end, it will be used in the future to pay for the unlock transaction by the bridge
12. Validator Program ID, use `va1udM9Gg22vcEbcuu4bsAu6tipRR8KSCHTGbaAhYQk`
13. System instructions account, use `Sysvar1nstructions1111111111111111111111111`
14. System rent account, use `SysvarRent111111111111111111111111111111111`
15. SPL Token Program ID, use `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`
16. System Program ID, use `11111111111111111111111111111111`

##### Signature verification instruction

The heavy lifting of the signature verification is handled by the system Secp256k1 Program (`KeccakSecp256k11111111111111111111111111111`). It should be added to the same transaction as the `unlock` instruction and its index in the transaction used in `secp_instruction_index` field. The data for the Secp256k1 Program instruction is already prepared by the Allbridge API, you can use the data returned in the `signature` field of the `sign` method call.

## Utility endpoints

### List supported tokens

To get the list of supported tokens you need to call:

```http request
GET https://allbridgeapi.net/token-info
```

Example response:

```json5
{
  "POL": {  // Blockchain ID
    "confirmations": 35, // Number of confirmations required to obtain signature         
    "tokens": [ // List of tokens available in the current blockchain
      {
        "address": "0x7DfF46370e9eA5f0Bad3C4E29711aD50062EA7A4", // Token address in the current blockchain
        "minFee": "10000000000000000", // Bridge min fee with token precision
        "tokenSource": "SOL", // Main token blockchain ID
        "tokenSourceAddress": "0x069b8857feab8184fb687f634618c035dac439dc1aeb3b5598a0f00000000001", // Main token address (32 bytes hex, zeros at the end)
        "isBase": false, // If this token is main asset (ETH for Ethereum, SOL for Solana)
        "isWrapped": true, // If token is wrapped by bridge 
        "precision": 18, // Token precision (decimals)
        "symbol": "SOL", // Token symbol
        "swapInfo": null, // Additional info for swap of Solana By Saber
        "logo": "https://allbridge-assets.web.app/logo/POL/0x7DfF46370e9eA5f0Bad3C4E29711aD50062EA7A4.png" // Token logo
      }
    ]
  }
}
```

Blockchain ID is a string constant uniquely identifying supported blockchain (defined [here](#blockchain-ids)).

The combination `tokenSource` and `tokenSourceAddress` is a unique token identifier within the bridge.

### Check recipient address
To check recipient address call:
```http request
GET https://allbridgeapi.net/check/{blockchainId}/address/{address}
```
Response example:

```json5
{
  "result": true, // Is a valid address
  "status": "OK" // Address status
}
```

Possible address statuses:

- `OK` Address is valid
- `INVALID` Invalid address
- `FORBIDDEN` Address is in forbidden list
- `UNINITIALIZED` Address is not initialized (only for Solana)
- `CONTRACT_ADDRESS` Contract address (only for Solana)

### Check recipient token balance
To check recipient token balance on the bridge you need to call server method:
```http request
GET https://allbridgeapi.net/check/{blockchainId}/balance/{tokenSource}/{tokenSourceAddress}
```

```json5
{
  "balance": "25807.385522832", // Current token balance on the destination blockchain. Could be zero if the token is wrapped.
  "isWrapped": false, // Is token wrapped by the bridge
  "tokenAddress": "So11111111111111111111111111111111111111112" // Token address on the destination blockchain
}
```

## Fee calculation

Allbridge fee can be of two types

- **Dynamic percentage fee**. Usually the amount of the fee is `0.3%` of the amount transferred and the fee is deducted on the sending blockchain. Users can reduce their fees by [staking](https://stake.allbridge.io) ABR token. After staking, the user receives stake pool shares in the form of xABR tokens and the balance of those tokens on the sending address is a basis for the fee reduction. Resulting fee is:

<img src="https://render.githubusercontent.com/render/math?math=\Large FEE = \Large \frac{1}{\frac{xABR\_USER}{xABR\_TOTAL} \times MULTIPLIER %2B \frac{1}{BASE\_FEE\_RATE}}" style="margin-bottom: 15px;"/>

- `FEE` is a resulting fee
- `xABR_USER` is user balance in xABR
- `xABR_TOTAL` is a total xABR minted supply
- `MULTIPLIER` is a constant, which can be changed on each particular blockchain, basically it is a measure of users' stake effect on the fee. The larger the multiplier the smaller stake is required to reduce the fee significantly
- `BASE_FEE_RATE` is a default fee rate (`0.3%`)

If user xABR balance is zero the effective fee is exactly equal to the base fee rate. And the higher the balance is the lower the fee, however it is never zero.

- **Static fee** is the minimum fee charged for the transfer. It is set to each asset individually and typically is around `$0.50`. If the dynamic percentage fee (multiplied by the transfer amount) gets smaller than the static fee then the static fee
gets charged instead. As a special case, when the base fee rate of the dynamic percentage fee is set to `0`, then the static fee is the one always charged for all the transfers.

## Constants

### Blockchain IDs

- `AVA` Avalanche
- `BSC` Binance Smart Chain
- `CELO` Celo
- `ETH` Ethereum
- `FTM` Fantom
- `HECO` Huobi ECO Chain
- `POL` Polygon
- `SOL` Solana
- `TRA` Terra
