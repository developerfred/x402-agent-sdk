# Supported Blockchains

The x402-agent-sdk supports multiple blockchain networks for payments.

## EVM Chains

All EVM-compatible networks are supported through the Ethereum standard.

### Network List

| Chain | Chain ID | Native Token | USDC Address | Status |
|-------|----------|--------------|--------------|--------|
| Ethereum | 1 | ETH | 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 | ✅ |
| Polygon | 137 | MATIC | 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 | ✅ |
| Base | 8453 | ETH | 0x833589fCD6eDb6E08f4c7C32D4f71B54bdA02913 | ✅ |
| Arbitrum | 42161 | ETH | 0xaf88d065e77c8cC2239327C5EDb3A432268e5831 | ✅ |
| Optimism | 10 | ETH | 0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85 | ✅ |
| Avalanche | 43114 | AVAX | 0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E | ✅ |
| BSC | 56 | BNB | 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d | ✅ |
| Celo | 42220 | CELO | 0x765DE816845861e75A25fCA122bb6898B8B1272a |
| Sepolia (Testnet) | 11155111 | SepoliaETH | 0x... |
| Amoy (Testnet) | 80002 | MATIC | 0x... |

### Usage (Rust)

```rust
use x402_agent_sdk::{EvmNetwork, EvmPayment, EvmToken};

// Ethereum
let payment = EvmPayment::new(
    EvmNetwork::Ethereum,
    "0x742d...",
    EvmToken::USDC,
    100,
);

// Polygon
let payment = EvmPayment::new(
    EvmNetwork::Polygon,
    "0x742d...",
    EvmToken::USDC,
    100,
);

// Custom EVM chain
let payment = EvmPayment::new(
    EvmNetwork::from_chain_id(1234),
    "0x742d...",
    EvmToken::USDC,
    100,
);
```

### Usage (TypeScript)

```typescript
import { EvmNetwork, EvmToken } from '@x402/sdk/chains/evm';

const payment = {
  network: 'eip155:1',      // Ethereum
  token: 'USDC',
  amount: '100',
  recipient: '0x742d...',
};
```

## Solana

### Network List

| Network | Chain ID | USDC Address | Status |
|---------|----------|--------------|--------|
| Mainnet | 101 | EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ | ✅ |
| Devnet | 102 | - | ✅ |
| Testnet | 103 | - | ✅ |

### Usage (Rust)

```rust
use x402_agent_sdk::{SolanaNetwork, SolanaPayment, SolanaToken};

let payment = SolanaPayment::new(
    SolanaNetwork::Mainnet,
    "Gk7sh8...",           // recipient address
    SolanaToken::USDC,      // or SolanaToken::SOL, SolanaToken::USDT
    100,                    // amount in lamports (1e-6 SOL)
);
```

### Token Support

- `SolanaToken::SOL` - Native SOL
- `SolanaToken::USDC` - USD Coin (SPL)
- `SolanaToken::USDT` - Tether (SPL)

## Stacks

### Network List

| Network | Chain ID | Native Token | Status |
|---------|----------|-------------|--------|
| Mainnet | 0x01 | STX | ✅ |
| Testnet | 0x80000000 | STX | ✅ |
| Devnet | 0x80000001 | STX | ✅ |

### Usage (Rust)

```rust
use x402_agent_sdk::{StacksNetwork, StacksPayment, StacksToken};

let payment = StacksPayment::new(
    StacksNetwork::Mainnet,
    "SP123...",              // recipient STX address
    StacksToken::STX,       // or StacksToken::SIP010 for FTs
    100,                    // micro-STX (1e-6 STX)
);
```

### Token Support

- `StacksToken::STX` - Native STX
- `StacksToken::SIP010` - Fungible tokens (SIP-010 standard)

## Tempo (Stripe + Paradigm)

### Overview

Tempo is a Layer 1 blockchain built by Stripe and Paradigm specifically for payments. It features:

- **Native Stripe Integration**: Accept fiat and crypto payments
- **Machine Payments Protocol (MPP)**: AI agent payment standard
- **Sessions**: Pre-authorize spending caps (OAuth for money)
- **100K TPS**: High throughput with sub-second finality
- **Low Fees**: Sub-millidollar transaction costs

### Network List

| Network | Chain ID | Features | Status |
|---------|----------|----------|--------|
| Mainnet | 5000 | Full production | ✅ |
| Testnet | 5001 | Testing | ✅ |
| Devnet | 5002 | Local development | ✅ |

### Supported Tokens

| Token | Symbol | Decimals | Status |
|-------|--------|----------|--------|
| USD Coin | USDC | 6 | ✅ |
| Tether | USDT | 6 | ✅ |
| Euro Coin | EUROC | 2 | ✅ |
| British Pound | GBPC | 2 | ✅ |

### Usage (Rust)

```rust
use x402_agent_sdk::{TempoNetwork, TempoPayment, TempoToken};

let payment = TempoPayment::new(
    TempoNetwork::Mainnet,
    "0x742d...",            // recipient address
    TempoToken::USDC,      // payment token
    100,                    // amount (cents for USDC)
);

// Convert to x402 PaymentRequired
let pr = payment.to_payment_required();
```

### Machine Payments Protocol (MPP) Sessions

Tempo introduces **sessions** - a revolutionary concept for AI agent payments:

```rust
use x402_agent_sdk::TempoSession;

// Create a session for an AI agent
let mut session = TempoSession {
    session_id: "sess_abc123".to_string(),
    authorized_spender: "0xagent...".to_string(),
    max_amount: 1000000,  // $10,000 cap
    spent: 0,
    expires_at: now + 3600,  // 1 hour
    token: TempoToken::USDC,
};

// Agent can make multiple payments within the session
if session.can_spend(100) {
    session.record_payment(100)?;
}
```

### Stripe Integration

Tempo has native Stripe integration:

```rust
use x402_agent_sdk::TempoStripeConfig;

let config = TempoStripeConfig {
    stripe_publishable_key: "pk_...".to_string(),
    merchant_id: "merchant_123".to_string(),
    enable_fiat: true,    // Accept credit cards, etc.
    enable_crypto: true,  // Accept USDC, etc.
};
```

### Usage (TypeScript)

```typescript
import { 
  TempoNetwork, 
  TempoToken, 
  TempoPayment,
  TempoSessionManager 
} from '@x402/sdk/adapters/tempo';

const payment = new TempoPayment({
  network: TempoNetwork.Mainnet,
  recipient: '0x742d...',
  token: TempoToken.USDC,
  amount: 100n,
  description: 'API access',
});

// MPP Sessions
const manager = new TempoSessionManager();
const session = manager.createSession(
  '0xagent...',   // authorized spender
  1000000n,       // max amount
  TempoToken.USDC,
  3600            // expires in 1 hour
);
```

## Protocol Comparison

| Feature | x402 (Exact) | MPP (Sessions) |
|---------|-------------|----------------|
| Payment Model | One-shot | One-shot + Streaming |
| Pre-authorization | ❌ | ✅ |
| Spending Caps | ❌ | ✅ |
| Fiat Support | Via facilitator | Native (Stripe) |
| Use Case | Single payments | API subscriptions |

## Adding Custom Networks

### EVM

```rust
use x402_agent_sdk::EvmNetwork;

let custom = EvmNetwork::from_chain_id(12345)
    .with_token("USDC", "0xcustom...");
```

### Custom Payment Schemes

```rust
use x402_agent_sdk::PaymentRequired;

let pr = PaymentRequired::new(
    "v1-custom-exact",     // your scheme
    "custom:1",            // your network
    "CUSTOM",              // your token
    "100",                 // amount
    "0xcustom...",        // recipient
);
```
