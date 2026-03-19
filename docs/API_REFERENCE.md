# API Reference

## Rust Core Library

### Payment Types

#### PaymentRequired

Represents the payment requirements returned to clients.

```rust
use x402_agent_sdk::PaymentRequired;

let pr = PaymentRequired::new(
    "v1-eip155-exact",    // scheme
    "eip155:1",           // network  
    "USDC",               // payment token
    "100",                // max amount
    "0x742d...",         // recipient
);
```

**Fields:**
- `version: u32` - Protocol version (always 1)
- `scheme: String` - Payment scheme (e.g., "v1-eip155-exact")
- `network: String` - Network identifier (e.g., "eip155:1")
- `payment_token: String` - Token address or symbol
- `max_amount: String` - Maximum payment amount
- `recipient: String` - Payment recipient address
- `expires_at: Option<DateTime<Utc>>` - Expiration time
- `description: Option<String>` - Payment description
- `salt: Option<String>` - Random salt for uniqueness

**Methods:**
- `new(scheme, network, token, amount, recipient) -> Self`
- `with_expiry(dt) -> Self` - Set expiration
- `with_description(desc) -> Self` - Set description
- `to_header_value() -> Result<String, X402Error>` - Convert to HTTP header
- `from_header_value(value) -> Result<Self, X402Error>` - Parse from header

#### PaymentToken

Represents a payment token sent by the client.

```rust
use x402_agent_sdk::PaymentToken;

let token = PaymentToken::decode(encoded)?;
// or
let token = PaymentToken::new(
    "0xabc...",  // transaction hash
    "0xdef...",  // recipient
    "100",       // amount
    "solana:101", // network
);
```

**Methods:**
- `decode(encoded: &str) -> Result<Self, X402Error>`
- `encode(&self) -> Result<String, X402Error>`
- `is_expired() -> bool`

### Credential Types

#### Credential

W3C Verifiable Credential for agent identity.

```rust
use x402_agent_sdk::{Credential, CredentialBuilder};

let credential = CredentialBuilder::new(
    "https://issuer.example.com",  // issuer
    "agent-123",                  // subject
)
.claim("role", serde_json::json!("admin"))
.claim("permissions", serde_json::json!(["read", "write"]))
.expires_at(chrono::Utc::now() + chrono::Duration::hours(24))
.build();
```

### Chain Integrations

#### EVM Chains

```rust
use x402_agent_sdk::{
    EvmNetwork, EvmPayment, EvmToken,
    EvmNetwork::{Ethereum, Polygon, Base},
};

let payment = EvmPayment::new(
    Ethereum,                          // network
    "0x742d...",                     // recipient
    EvmToken::USDC,                   // token
    100,                              // amount (in smallest unit)
);

// Convert to PaymentRequired
let pr = payment.to_payment_required();
```

**Supported Networks:**
- `Ethereum` - Chain ID 1
- `Polygon` - Chain ID 137
- `Base` - Chain ID 8453
- `Arbitrum` - Chain ID 42161
- `Optimism` - Chain ID 10
- `Avalanche` - Chain ID 43114
- `BSC` - Chain ID 56
- `Celo` - Chain ID 42220

**Supported Tokens:**
- `USDC` - USD Coin
- `USDT` - Tether
- `Native` - Chain's native token (ETH, MATIC, etc.)

#### Solana

```rust
use x402_agent_sdk::{
    SolanaNetwork, SolanaPayment, SolanaToken,
};

let payment = SolanaPayment::new(
    SolanaNetwork::Mainnet,
    "Gk7sh...",
    SolanaToken::USDC,
    100,  // lamports
);
```

#### Stacks

```rust
use x402_agent_sdk::{
    StacksNetwork, StacksPayment, StacksToken,
};

let payment = StacksPayment::new(
    StacksNetwork::Mainnet,
    "SP123...",
    StacksToken::SIP010,  // FT token
    100,
);
```

#### Tempo (Stripe + Paradigm)

```rust
use x402_agent_sdk::{
    TempoNetwork, TempoPayment, TempoToken, TempoSession,
};

let payment = TempoPayment::new(
    TempoNetwork::Mainnet,
    "0x742d...",
    TempoToken::USDC,
    100,
);

// MPP Sessions for streaming payments
let session = TempoSession {
    session_id: "sess_123".to_string(),
    authorized_spender: "0xabc...".to_string(),
    max_amount: 10000,
    spent: 0,
    expires_at: now + 3600,
    token: TempoToken::USDC,
};
```

### Security

#### InputValidator

```rust
use x402_agent_sdk::InputValidator;

InputValidator::validate_address("0x742d...", "ethereum")?;
InputValidator::validate_amount("100")?;
InputValidator::validate_network("eip155:1")?;
```

#### RateLimiter

```rust
use x402_agent_sdk::{RateLimiter, RateLimitConfig};

let limiter = RateLimiter::new(RateLimitConfig {
    max_requests: 100,
    window_seconds: 60,
});

limiter.check("user_address").await?;
```

### Error Types

```rust
use x402_agent_sdk::X402Error;

match error {
    X402Error::InvalidHeader(msg) => ...,
    X402Error::InvalidToken(msg) => ...,
    X402Error::TokenExpired(time) => ...,
    X402Error::SignatureInvalid(msg) => ...,
    X402Error::Payment(msg) => ...,
    X402Error::UnsupportedScheme(scheme) => ...,
    X402Error::NetworkError(msg) => ...,
    X402Error::ParseError(msg) => ...,
    X402Error::ConfigError(msg) => ...,
}
```

---

## TypeScript / JavaScript

### Server Usage

```typescript
import { X402Server } from '@x402/sdk/server';

const server = new X402Server({
  pricePerRequest: 100,
  scheme: 'v1-eip155-exact',
  network: 'eip155:1',
  paymentToken: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  recipient: '0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E',
});

// Express middleware
app.use(server.middleware());
```

### Client Usage

```typescript
import { X402Client } from '@x402/sdk/client';

const client = new X402Client({
  facilitatorUrl: 'https://facilitator.x402.rs',
  wallet: myWallet,
});

const response = await client.request('https://api.example.com/data');
```

### Tempo Adapter

```typescript
import { TempoPayment, TempoSessionManager } from '@x402/sdk/adapters/tempo';

const payment = new TempoPayment({
  network: TempoNetwork.Mainnet,
  recipient: '0x742d...',
  token: TempoToken.USDC,
  amount: 100n,
});

// MPP Sessions
const manager = new TempoSessionManager();
const session = manager.createSession(
  '0xabc...',  // authorized spender
  10000n,      // max amount
  TempoToken.USDC,
  3600         // expires in 1 hour
);
```

---

## Framework Adapters

### Express.js

```typescript
import express from 'express';
import { x402Middleware } from '@x402/sdk/adapters/express';

const app = express();

app.use(x402Middleware({
  pricePerRequest: 100,
  scheme: 'v1-eip155-exact',
  network: 'eip155:1',
  paymentToken: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  recipient: '0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E',
}));
```

### Fastify

```typescript
import Fastify from 'fastify';
import { x402Plugin } from '@x402/sdk/adapters/fastify';

const fastify = Fastify();
await fastify.register(x402Plugin, {
  pricePerRequest: 100,
  scheme: 'v1-eip155-exact',
  network: 'eip155:1',
  paymentToken: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  recipient: '0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E',
});
```

### Python (FastAPI)

```python
from x402_fastapi import X402Middleware, X402Config

config = X402Config(
    price_per_request=100,
    network="eip155:1",
    payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
)

app = FastAPI()
app.add_middleware(X402Middleware, config=config)
```

### Go (Gin)

```go
import "github.com/x402/adapters/go/gin"

router.Use(gin.Middleware(gin.Config{
    PricePerRequest: 100,
    Network:        "eip155:1",
    PaymentToken:   "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    Recipient:      "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
}))
```

### Rust (Axum)

```rust
use x402_agent_sdk::adapters::axum::{x402_middleware, AxumX402Config};

let config = AxumX402Config::new(
    100,
    "v1-eip155-exact",
    "eip155:1",
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "0x742d..."
).with_description("API Access");

Router::new()
    .route("/api/data", get(handler))
    .layer(middleware::from_fn(move |req, next| {
        x402_middleware(config.clone(), req, next)
    }));
```
