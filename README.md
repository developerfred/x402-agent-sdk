# x402 Agent SDK

<p align="center">
  <a href="https://github.com/Developerfred/x402-agent-sdk/actions/workflows/ci.yml">
    <img src="https://github.com/Developerfred/x402-agent-sdk/actions/workflows/ci.yml/badge.svg" alt="CI">
  </a>
  <a href="https://crates.io/crates/x402-agent-sdk">
    <img src="https://img.shields.io/crates/v/x402-agent-sdk" alt="Crate">
  </a>
  <a href="https://www.npmjs.com/package/@x402/sdk">
    <img src="https://img.shields.io/npm/v/@x402/sdk" alt="NPM">
  </a>
  <a href="https://discord.gg/x402">
    <img src="https://img.shields.io/discord/123456789" alt="Discord">
  </a>
</p>

> Transform any API into an x402 payment-enabled API in minutes. Built for AI agents and autonomous systems.

## Why x402?

The HTTP 402 "Payment Required" status code has been unused for 30 years. x402 finally puts it to work, enabling:

- **Agent-to-Agent Payments**: AI agents can autonomously pay for API access
- **Micropayments**: Pay per request, no subscriptions required
- **No Accounts**: Cryptographic wallet-based authentication
- **Multi-Chain**: Support for EVM, Solana, and Stacks

## Features

- 🌐 **Multi-Chain Support**: Ethereum, Polygon, Base, Arbitrum, Optimism, Avalanche, BSC, Celo, Solana, Stacks, Tempo
- ⚡ **Tempo.xyz Integration**: Stripe-backed blockchain with Machine Payments Protocol (MPP)
- 🔐 **W3C Verifiable Credentials**: Built-in credential generation for agent identity
- 🛡️ **Security First**: Input validation, rate limiting, address sanitization
- ⚡ **Fast**: Rust core with WASM bindings for browser/Node.js
- 🔌 **Framework Adapters**: Express, Fastify, Next.js, Axum, Django, Flask, Gin, Fiber, Spring Boot
- 📦 **TypeScript**: Full type definitions included
- 💳 **MPP Sessions**: OAuth for money - pre-authorize spending caps for AI agents

## Quick Start

### Rust

```toml
# Cargo.toml
[dependencies]
x402-agent-sdk = "0.1"
```

```rust
use x402_agent_sdk::{
    AgentServer,
    EvmNetwork,
    EvmPayment,
    EvmToken,
};

let server = AgentServer::new(
    100,                      // price per request
    "v1-eip155-exact",       // payment scheme
    "eip155:1",             // network (Ethereum)
    "USDC",                 // payment token
    "0x742d...",           // recipient address
);

// Create payment requirement
let pr = server.create_payment_required(Some("Premium API Access"));
```

### TypeScript / Deno

```typescript
import { X402Server, X402Client } from "https://deno.land/x/x402/mod.ts";

// Server
const server = new X402Server(
  100,
  "v1-eip155-exact",
  "eip155:1",
  "USDC",
  "0x742d..."
);

// Client
const client = new X402Client({
  facilitatorUrl: "https://facilitator.x402.rs"
});

const response = await client.request("https://api.example.com/data");
```

### Node.js / Browser (WASM)

```javascript
import init, { WasmPaymentRequired } from 'x402-agent-sdk';

await init();

const pr = new WasmPaymentRequired(
  "v1-eip155-exact",
  "eip155:1",
  "USDC",
  "100",
  "0x742d..."
);
```

## Supported Blockchains

### EVM Chains

| Chain | Chain ID | Native Token | USDC Address |
|-------|---------|--------------|--------------|
| Ethereum | 1 | ETH | 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 |
| Polygon | 137 | MATIC | 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 |
| Base | 8453 | ETH | 0x833589fCD6eDb6E08f4c7C32D4f71B54bdA02913 |
| Arbitrum | 42161 | ETH | 0xaf88d065e77c8cC2239327C5EDb3A432268e5831 |
| Optimism | 10 | ETH | 0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85 |
| Avalanche | 43114 | AVAX | 0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E |
| BSC | 56 | BNB | 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d |

### Solana

| Network | Chain ID | Token Address |
|---------|----------|--------------|
| Mainnet | 101 | EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ (USDC) |
| Devnet | 102 | - |
| Testnet | 103 | - |

### Stacks

| Network | Chain ID | Token |
|---------|----------|-------|
| Mainnet | 0x01 | STX |
| Testnet | 0x80000000 | STX |

### Tempo (Stripe + Paradigm)

| Network | Chain ID | Features |
|---------|----------|----------|
| Mainnet | 5000 | Stripe integration, MPP Sessions |
| Testnet | 5001 | Testing |
| Devnet | 5002 | Local development |

**Tempo** is Stripe and Paradigm's blockchain for payments:
- Native Stripe integration (accept fiat + crypto)
- Machine Payments Protocol (MPP) for AI agents
- Sessions: Pre-authorize spending caps for streaming payments
- 100K TPS, sub-second finality

## Framework Integration

### Express.js

```typescript
import express from "express";
import { x402Middleware } from "x402-agent-sdk/adapters/express";

const app = express();

app.use(x402Middleware({
  pricePerRequest: 100,
  scheme: "v1-eip155-exact",
  network: "eip155:1",
  paymentToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  recipient: "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
}));

app.get("/api/data", (req, res) => {
  res.json({ data: "Paid content!" });
});
```

### Fastify

```typescript
import Fastify from "fastify";
import { x402Plugin } from "x402-agent-sdk/adapters/fastify";

const fastify = Fastify();

await fastify.register(x402Plugin, {
  pricePerRequest: 100,
  scheme: "v1-eip155-exact",
  network: "eip155:1",
  paymentToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  recipient: "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
});
```

### Next.js

```typescript
// middleware.ts
import { x402Middleware } from "x402-agent-sdk/adapters/nextjs";

export function middleware(request: NextRequest) {
  return x402Middleware({
    pricePerRequest: 100,
    scheme: "v1-eip155-exact",
    network: "eip155:1",
    paymentToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    recipient: "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
  })(request);
}
```

### Axum (Rust)

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

### Python

#### FastAPI

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

#### Flask

```python
from x402_flask import X402, X402Config

x402 = X402(app, config=X402Config(
    price_per_request=100,
    network="eip155:1",
    payment_token="0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
))
```

#### Django

```python
# settings.py
MIDDLEWARE = [
    'x402_django.X402Middleware',
]

X402_CONFIG = {
    'price_per_request': 100,
    'network': 'eip155:1',
    'payment_token': '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
    'recipient': '0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E',
}
```

### Go

#### Gin

```go
import "x402/adapters/go/gin"

router.Use(gin.Middleware(gin.Config{
    PricePerRequest: 100,
    Network:        "eip155:1",
    PaymentToken:   "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    Recipient:      "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
}))
```

### Java

#### Spring Boot

```java
@Configuration
public class X402Config {
    @Bean
    public X402Config x402Config() {
        X402Config config = new X402Config();
        config.setPricePerRequest(100);
        config.setNetwork("eip155:1");
        config.setPaymentToken("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        config.setRecipient("0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E");
        return config;
    }
}
```

### C# /.NET

```csharp
var builder = WebApplication.CreateBuilder(args);
builder.Services.AddX402(options => {
    options.PricePerRequest = 100;
    options.Network = "eip155:1";
    options.PaymentToken = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    options.Recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E";
});
```

### Ruby

#### Rails

```ruby
# Gemfile
gem 'x402_rails'

# config/initializers/x402.rb
Rails.application.config.x402.price_per_request = 100
Rails.application.config.x402.network = "eip155:1"
Rails.application.config.x402.payment_token = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
Rails.application.config.x402.recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
```

### PHP

#### Laravel

```php
# app/Http/Kernel.php
protected $middleware = [
    \App\Http\Middleware\X402Middleware::class,
];
```

## Payment Flow

```
┌──────────┐    GET /data     ┌────────────┐
│  Agent   │ ───────────────▶ │   Server   │
└──────────┘                  └──────┬─────┘
                                     │
                              ┌──────▼──────┐
                              │ No Payment?  │
                              │  402 Response│
                              │ + PaymentReq │
                              └──────┬──────┘
                                     │
       ┌─────────────────────────────┘
       │ 1. Parse PaymentRequired
       │ 2. Fund Wallet
       │ 3. Sign Payment
       ▼
┌──────────┐   GET /data      ┌────────────┐
│  Agent   │ + X-Payment-Token │   Server   │
│          │ ────────────────▶ │            │
└──────────┘                  └──────┬─────┘
                                     │
                              ┌──────▼──────┐
                              │ Verify Token │
                              │  Return Data │
                              └──────────────┘
```

## Security

```rust
use x402_agent_sdk::{InputValidator, RateLimiter, RateLimitConfig};

// Address validation
InputValidator::validate_address("0x742d...", "ethereum")?;  // ✅
InputValidator::validate_address("invalid", "ethereum")?;    // ❌

// Amount validation  
InputValidator::validate_amount("100")?;  // ✅
InputValidator::validate_amount("0")?;   // ❌

// Rate limiting
let limiter = RateLimiter::new(RateLimitConfig {
    max_requests: 100,
    window_seconds: 60,
});
limiter.check("user_address").await?;
```

## Environment Variables

```bash
# Server
X402_ENABLED=true
X402_NETWORK=eip155:1
X402_PAYMENT_TOKEN=0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
X402_RECIPIENT=0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E

# Client
X402_FACILITATOR_URL=https://facilitator.x402.rs
```

## Development

```bash
# Build
cargo build

# Test
cargo test

# WASM Build
wasm-pack build --target web --out-dir pkg

# Documentation
cargo doc --open
```

## Resources

- [Documentation](./docs/DEVELOPER_GUIDE.md)
- [x402 Protocol Spec](https://www.x402.org)
- [Examples](./examples/)
- [Discord](https://discord.gg/x402)

## License

MIT or Apache-2.0
