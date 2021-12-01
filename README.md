# Allbridge

Allbridge documentation for EVN. It describes how to transfer assets from one blockchain to another using Allbridge.

## Getting available token info

To get info about available tokens you need to call a server method:

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

Blockchain IDs:
```
    AVA - Avalanche
    BSC - Binance Smart Chain
    CELO - Celo
    ETH - Ethereum
    FTM - Fantom
    HECO - Huobi ECO Chain
    POL - Polygon
    SOL - Solana
    TRA - Terra
```
For identification token use `tokenSource` `tokenSourceAddress` pair

## Check destination address
To check destination address call:
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

```
OK - Address is valid
INVALID - Invalid address
FORBIDDEN - Address is in forbidden list
UNINITIALIZED - Address is not itialized (only for Solana)
CONTRACT_ADDRESS - Contract address (only for Solana)
```

## Check destination token balance
To check destination token balance on the bridge you need to call server method:
```http request
GET https://allbridgeapi.net/check/{blockchainId}/balance/{tokenSource}/{tokenSourceAddress}
```

```json5
{
  "balance": "25807.385522832", // Current token balance on the destination blockchain. Could be zero if token is wrapped.
  "isWrapped": false, // Is token wrapped by the bridge
  "tokenAddress": "So11111111111111111111111111111111111111112" // Token address on the destination blockchain
}
```
## Create lock

Call `lock` method

```solidity
    function lock(uint128 lockId, address tokenAddress, bytes32 recipient, bytes4 destination, uint256 amount)
```

```
    lockId - Random uint128 number. First byte must be 0x01 !
    tokenAddress - Token address
    recipient - Recipient address as 32 bytes (zeros at the end)
    destination - Blockchain ID as 4 bytes (UTF8, zeros at the end)
    amount - Amount of token to transfer
```

For native tokens use another method:
```solidity
    function lockBase(uint128 lockId, address wrappedBaseTokenAddress, bytes32 recipient, bytes4 destination) payable
```

```
    lockId - Random uint128 number. First byte must be 0x01 !
    wrappedBaseTokenAddress - Wrapped token address (WETH address)
    recipient - Recipient address as 32 bytes (zeros at the end)
    destination - Blockchain ID as 4 bytes (UTF8, zeros at the end)
```

## Get signature

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

## Unlock method 
All parameters for unlock is returned by `sign` method    

```solidity
function unlock(uint128 lockId, address recipient, uint256 amount, bytes4 lockSource, bytes4 tokenSource, bytes32 tokenSourceAddress, bytes calldata signature)
```

```
lockId - The same as in the sign method result.
recipient - Transformed to valid address in the destination blockchain.
amount - Amount in system precision (9). Exect the same as in the sign method result.
lockSource - Transfer source blockchain ID (4 bytes, UTF8, zeros at the end).
tokenSource - Token source blockchain ID (4 bytes, UTF8, zeros at the end).
tokenSourceAddress - Token source address. Exect the same as in the sign method result.
signature - Signature for unlock. Exect the same as in the sign method result.
```


