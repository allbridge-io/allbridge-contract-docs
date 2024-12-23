***!!! Information below is about Allbridge Classic !!!***

***!!! If you want to work with Allbridge Core stablecoin bridge please visit [https://github.com/allbridge-io/allbridge-core-docs](https://github.com/allbridge-io/allbridge-core-docs) or [https://github.com/allbridge-io/allbridge-core-js-sdk](https://github.com/allbridge-io/allbridge-core-js-sdk) !!!***

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
- [Multisig bridge](#multisig-bridge)

## Main transfer flow

### Lock tokens

#### EVM

Call `lock` method:

```solidity
    function lock(uint128 lockId, address tokenAddress, bytes32 recipient, bytes4 destination, uint256 amount)
```

- `lockId` a random 16-byte value identifying the transfer within the bridge, with first byte is used as a bridge version (must be `0x01`)
- `tokenAddress` is an address of the token you want to send
- `recipient` recipient address as 32 bytes (zeros at the end if receiving blockchain has addresses shorter than 32 bytes)
- `destination` [Blockchain ID](#blockchain-ids) as 4 bytes (UTF8, zeros at the end if shorter than 4 bytes)
- `amount` amount to lock on EVM side

For native tokens use a different method:
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
- `amount` amount (including fee) to lock on the Solana side
- `lock_id` a random 16-byte value identifying the transfer within the bridge, with the first byte is used as a bridge version (must be `0x01`)

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

#### XRPL

On XRPL locking tokens is a simple transfer to the bridge address `r4w1LrneWZqX5RrgFPx2gto66dwo2Zymqy`. Bridging destination is encoded in the following fields:

- `DestinationTag` is a destination blockchain encoded in 4-byte hex with trailing zeroes. For example, `1112752896` is `0x42534300` or `BSC`
- `InvoiceID` should be set to the recipient's address (32 bytes with trailing zeroes)

Here is a [sample transaction](https://livenet.xrpl.org/transactions/3201416350CA9D6B63684E49C7F2BF99146802DBB489A56FC5F5EB1728C69D5B/raw).

Alternatively you can call Allbridge server to prepare transaction and return QR code for the wallet to scan by sending `POST` to `https://xrpl.allbridgeapi.net/xumm/transaction` or `https://xrpl.allbridgeapi.net/solo/transaction` with the following JSON:

- `from` sender address, for example `rHfGE9y7MSfJc4pEG3mua7tvAqoE4jsCPh`
- `amount` amount to send, integer as string, `10000000`
- `destination` destination blockchain, string, `BSC`
- `recipient` recipient address, 32 byte hex as string, `081A16070B02181B1B17171E100D0A170E1D111B000000000000000000000000`
- `tokenAddress` token minter address, string, `r3kCiZTA9N7RjK2bYCmJSoFcnDQs95apd7`
- `symbol` token symbol, string, `aeUSDC`

#### Tezos

Call `lock_asset` method:

```
type lock_asset_t       is [@layout:comb] record[
  chain_id                : chain_id_t;
  lock_id                 : lock_id_t;
  token_source            : bytes;
  token_source_address    : bytes;
  amount                  : nat;
  recipient               : bytes;
]
```

- `chain_id` destination [Blockchain ID](#blockchain-ids) as 4 bytes (UTF8, zeros at the end if shorter than 4 bytes)
- `lock_id` a random 16-byte value identifying the transfer within the bridge, with first byte is used as a bridge version (must be `01`)
- `token_source` token source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end)
- `token_source_address` is a source address of the token you want to send (32 bytes)
- `amount` amount to lock on Tezos side
- `recipient` recipient address as 32 bytes (zeros at the end if receiving blockchain has addresses shorter than 32 bytes)

`If you need to send native token, you have to attach to the transaction the same amount as in the arguments.`


### Get signature

Allbridge API endpoint to get transaction details and signature using transaction lock ID
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

All parameters for unlock are returned by the Allbridge API `sign` method (previous step)

```solidity
function unlock(uint128 lockId, address recipient, uint256 amount, bytes4 lockSource, bytes4 tokenSource, bytes32 tokenSourceAddress, bytes calldata signature)
```

- `lockId` Lock ID value of the initial lock on Blockchain #1 (returned by the `sign` Allbridge API call)
- `recipient` Recipient address returned by the `sign` API call and formatted as EVM address (first 20 bytes)
- `amount` Amount in the internal precision of the bridge (9 digits), use the same amount as returned by the `sign` Allbridge API call
- `lockSource` Transfer source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end), `source` field in `sign` response
- `tokenSource` Token source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end), `tokenSource` field in `sign` response
- `tokenSourceAddress` Token source address, `tokenSourceAddress` field in `sign` response
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

The heavy lifting of the signature verification is handled by the system Secp256k1 Program (`KeccakSecp256k11111111111111111111111111111`). It should be added to the same transaction as the `unlock` instruction. Also the position this instruction appears in the transaction (its index) should be used in `secp_instruction_index` parameter of the `unlock` instruction. The data for the Secp256k1 Program instruction is already prepared by the Allbridge API, you can use the data returned in the `signature` field of the `sign` method call.

#### XRPL

##### Trust Line

To receive tokens on XRPL (if it is not XRPL) a trust line has to be established. You can do it yourself or use Allbridge server to prepare transaction for the user. Send `POST` to `https://xrpl.allbridgeapi.net/xumm/create-line` or `https://xrpl.allbridgeapi.net/solo/create-line` with the following JSON:

- `userAddress` Recipient address, for example `rHfGE9y7MSfJc4pEG3mua7tvAqoE4jsCPh`
- `tokenAddress` Token minter address, `r3kCiZTA9N7RjK2bYCmJSoFcnDQs95apd7`
- `symbol` Token symbol on XRPL, for example `UST`

##### Unlock

Send `POST` to `https://xrpl.allbridgeapi.net/unlock` with the following JSON object:

```json5
{
  "lockId": "2628534210935351210556389661603063554", // Lock ID received by the call to the signer
  "recipient": "0x7a7d5401dd19f6a60c7b24af9861b22e593d52f518ded5714000000000000000", // Recipient address in hex
  "amount": "3000000000", // Integer amount as string
  "source": "TRA", // Transaction source address
  "tokenSource": "TRA", // Token source address
  "tokenSourceAddress": "0x0e151a1706190000000000000000000000000000000000000000000000000000", // Token minter/contract address on the original chain
  "signature": "0x85c5e8613985a01db6ff77c61b1a59a0e45630755de045dd…9291b70abe53647865cee3224df76d62936d795e2dcb8601c" // Signature returned by the signer
}
```

#### Tezos

All parameters for unlock are returned by the Allbridge API `sign` method (previous step)

Call `unlock_asset` method:

```
type unlock_asset_t     is [@layout:comb] record[
  lock_id                 : lock_id_t;
  recipient               : address;
  amount                  : nat;
  chain_from_id           : chain_id_t;
  token_source            : bytes;
  token_source_address    : bytes;
  signature               : signature;
]
```

- `lock_id` Lock ID value of the initial lock on Blockchain #1 (returned by the `sign` Allbridge API call)
- `recipient` Recipient Tezos address (needs to be transformed from hex format encodePubKey, first 22 bytes)
- `amount` Amount in the internal precision of the bridge (9 digits), use the same amount as returned by the `sign` Allbridge API call
- `chain_from_id` Source [Blockchain ID](#blockchain-ids) as 4 bytes (UTF8, zeros at the end if shorter than 4 bytes)
- `token_source` Receiving token source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end)
- `token_source_address` Receiving token source address (32 bytes), use the same value as returned by the `sign` method
- `signature` Signature data, use the same value as returned by the `sign` method


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

### Ethereum

On Ethereum we always charge a minimum fee (`minFee` in token list).

### XRPL

On XRPL we charge `0.1%` fee unless it is smaller than `minFee`. If it is smaller we charge `minFee` instead.

### Other chains

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
- `TEZ` Tezos

### Tezos types
- `type lock_id_t is bytes`
- `type chain_id_t is bytes`

# Multisig bridge

## Intro

The Stellar integration in the Allbridge ecosystem operates using a multisig bridge mechanism, which requires two signatures for transferring assets.

### Lock tokens

#### Stellar

On Stellar, locking tokens is a simple transfer to the bridge's MuxedAccount, which depends on the destination chain, with the recipient address specified in the memo.
To build a correct transaction, you could call a POST method https://stellar.allbridgeapi.net/wallet/transaction with JSON body:

- `from` sender address, for example `GDL27JZFDPBXX7B4DTWPSEWRFHGTAQM6HK365M3J6LVAOBY6VCEUGRCU`
- `amount` amount to send, integer as string, `10000000`
- `destination` destination blockchain, string, `BSC`
- `recipient` recipient address, 32 byte hex as string, `081A16070B02181B1B17171E100D0A170E1D111B000000000000000000000000`
- `tokenAddress` token minter address, string, `GALLBRBQHAPW5FOVXXHYWR6J4ZDAQ35BMSNADYGBW25VOUHUYRZM4XIL`
- `symbol` token symbol, string, `aeUSDC`

It is very simmilar to XRPL.

#### Other chains
Locking on other chains is similar to the common lock flow.

To encode and decode Stellar address to 32byte you can use [encodeEd25519PublicKey](https://stellar.github.io/js-stellar-sdk/StrKey.html#.encodeEd25519PublicKey) and [decodeEd25519PublicKey](https://stellar.github.io/js-stellar-sdk/StrKey.html#.decodeEd25519PublicKey) methods from StellarSdk

### Get signature

Allbridge API endpoint to get transaction details and signature using transaction lock ID
```http request
GET https://stellar-info.allbridgeapi.net/sign/{transactionId}
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
  "signature": "012000000c0", // Signature to pass it to unlock method
  "secondarySignature": "012000000c1" // Secondary signature to pass it to unlock method
}
```
It is similar to a common request but returns an additional `secondarySignature` field


### Unlock tokens

#### EVM

All parameters for unlock are returned by the Allbridge API `sign` method (previous step)

```solidity
function unlock(uint128 lockId, address recipient, uint256 amount, bytes4 lockSource, bytes4 tokenSource, bytes32 tokenSourceAddress, bytes calldata signaturePrimary, bytes calldata signatureSecondary)
```

- `lockId` Lock ID value of the initial lock on Blockchain #1 (returned by the `sign` Allbridge API call)
- `recipient` Recipient address returned by the `sign` API call and formatted as EVM address (first 20 bytes)
- `amount` Amount in the internal precision of the bridge (9 digits), use the same amount as returned by the `sign` Allbridge API call
- `lockSource` Transfer source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end), `source` field in `sign` response
- `tokenSource` Token source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end), `tokenSource` field in `sign` response
- `tokenSourceAddress` Token source address, `tokenSourceAddress` field in `sign` response
- `signaturePrimary` Primary signature for unlock, pass the value received from the `sign` method call
- `signatureSecondary` Secondary signature for unlock, pass the value received from the `sign` method call 
It is similar to a common unlock but need additional `secondarySignature` field

#### Solana

The similar to common unlock, but needs additional secp instruction with secondary signature

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
    pub secp_instruction_index_1: u8,
    pub secp_instruction_index_2: u8,
}
```

- `lock_id` Lock ID value of the initial lock on Blockchain #1 (returned by the `sign` Allbridge API call)
- `lock_source` Transfer source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end)
- `amount` Amount in bridge internal precision (9 digits), use the same amount as returned by the `sign` Allbridge API call
- `token_source` Token source [Blockchain ID](#blockchain-ids) (4 bytes, UTF8, zeros at the end)
- `token_source_address` Token source address, use the same value as returned by the `sign` method
- `secp_instruction_index_1` Index of the primary signature verification instruction, typically `0` is used unless you need some instructions to be prior to it in the transaction
- `secp_instruction_index_2` Index of the secondary signature verification instruction, typically `1` is used unless you need some instructions to be prior to it in the transaction

#### Stellar

To receive tokens on Stellar (if it is not XLM) a trust line has to be established. You can do it yourself or use Allbridge server to prepare transaction for the user. Send `POST` to `https://stellar.allbridgeapi.net/wallet/create-line` with the following JSON:

- `userAddress` Recipient address, for example `GDL27JZFDPBXX7B4DTWPSEWRFHGTAQM6HK365M3J6LVAOBY6VCEUGRCU`
- `tokenAddress` Token minter address, `GALLBRBQHAPW5FOVXXHYWR6J4ZDAQ35BMSNADYGBW25VOUHUYRZM4XIL`
- `symbol` Token symbol on Stellar, for example `aeUSDC`


##### Unlock

Send `POST` to `https://stellar.allbridgeapi.net/unlock` with the following JSON object:

```json5
{
  "lockId": "2628534210935351210556389661603063554", // Lock ID received by the call to the signer
  "recipient": "0x7a7d5401dd19f6a60c7b24af9861b22e593d52f518ded5714000000000000000", // Recipient address in hex
  "amount": "3000000000", // Integer amount as string
  "source": "SOL", // Transaction source address
  "tokenSource": "SOL", // Token source address
  "tokenSourceAddress": "0x0e151a1706190000000000000000000000000000000000000000000000000000", // Token minter/contract address on the original chain
  "primarySignature": "0x85c5e8613985a01db6ff77c61b1a59a0e45630755de045dd…9291b70abe53647865cee3224df76d62936d795e2dcb8601c", // Primary signature returned by the signer
  "secondarySignature": "0x95c5e8613985a01db6ff77c61b1a59a0e45630755de045dd…9291b70abe53647865cee3224df76d62936d795e2dcb8601c" // Secondary signature returned by the signer
}
```

## Utility endpoints

The same, but instead of call https://allbridgeapi.net you should call https://stellar-info.allbridgeapi.net

For example:

```http request
GET https://stellar-info.allbridgeapi.net/token-info
```
