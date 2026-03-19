# x402 Agent SDK - Developer Guide

> Transform any API into an x402 payment-enabled API in minutes

## Quick Start

### Installation

```bash
# Rust
cargo add x402-agent-sdk

# Deno
deno add npm:@x402/agent-sdk
```

### Basic Usage (Deno/TypeScript)

```typescript
import { X402Server, X402Client, createSolanaPayment } from "./mod.ts";

// Server: Require payment for your API
const server = new X402Server(
  100,              // Price per request
  "v1-solana-exact", // Payment scheme
  "solana:101",     // Network (Solana Mainnet)
  "USDC",           // Payment token
  "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ" // Recipient address
);

// Client: Automatically pay for API access
const client = new X402Client({
  facilitatorUrl: "https://facilitator.x402.rs"
});

// Make a paid request
const response = await client.request("https://api.example.com/data");
```

---

## Supported Blockchains

### EVM Chains (Ethereum, Polygon, Base, etc.)

```typescript
import { createEvmPayment, EvmNetwork, EvmToken } from "./mod.ts";

// Ethereum + USDC
const ethPayment = createEvmPayment(
  EvmNetwork.Ethereum,
  "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
  EvmToken.USDC,
  100
);

// Polygon + USDT
const polygonPayment = createEvmPayment(
  EvmNetwork.Polygon,
  "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
  EvmToken.USDT,
  50
);

// Base + ETH
const basePayment = createEvmPayment(
  EvmNetwork.Base,
  "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
  EvmToken.ETH,
  1000
);
```

**Supported EVM Networks:**
- Ethereum Mainnet (chainId: 1)
- Polygon (chainId: 137)
- Base (chainId: 8453)
- Arbitrum (chainId: 42161)
- Optimism (chainId: 10)
- Avalanche (chainId: 43114)
- BSC (chainId: 56)
- Celo (chainId: 42220)
- Sepolia Testnet (chainId: 11155111)
- Amoy Testnet (chainId: 80002)

**Supported EVM Tokens:**
- ETH, WETH, USDC, USDT, DAI, WBTC

### Solana

```typescript
import { createSolanaPayment, SolanaNetwork, SolanaToken } from "./mod.ts";

const solanaPayment = createSolanaPayment(
  SolanaNetwork.Mainnet,
  "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
  SolanaToken.USDC,
  100
);
```

### Stacks

```typescript
import { createStacksPayment, StacksNetwork, StacksToken } from "./mod.ts";

const stacksPayment = createStacksPayment(
  StacksNetwork.Mainnet,
  "SP3K8BC0PPEVCV7NZ6QSRWPQ2ZQ9GFG3EEEGH1GB",
  StacksToken.STX,
  1000
);
```

---

## Framework Integrations

### Express.js

```typescript
import express from "express";
import { x402Middleware, paymentMiddleware } from "./adapters/express/index.ts";

const app = express();

// Simple middleware usage
app.use(x402Middleware({
  pricePerRequest: 100,
  scheme: "v1-eip155-exact",
  network: "eip155:1",
  paymentToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  recipient: "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
  description: "API Access"
}));

// Route-specific configuration
app.use(paymentMiddleware({
  "GET /api/data": {
    accepts: ["v1-eip155-exact", "v1-solana-exact"],
    description: "Premium data access",
    price: 50
  },
  "POST /api/analyze": {
    accepts: ["v1-eip155-exact"],
    description: "AI Analysis",
    price: 500
  }
}));

app.get("/api/data", (req, res) => {
  res.json({ data: "Your paid content here!" });
});
```

### Fastify

```typescript
import Fastify from "fastify";
import { x402Plugin, paymentMiddleware } from "./adapters/fastify/index.ts";

const fastify = Fastify({ logger: true });

await fastify.register(x402Plugin, {
  pricePerRequest: 100,
  scheme: "v1-eip155-exact",
  network: "eip155:1",
  paymentToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  recipient: "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
});

fastify.get("/api/data", async (request, reply) => {
  return { data: "Your paid content here!" };
});
```

### Next.js (App Router)

```typescript
// middleware.ts
import { x402Middleware } from "./adapters/nextjs/index.ts";

export function middleware(request: NextRequest) {
  return x402Middleware({
    pricePerRequest: 100,
    scheme: "v1-eip155-exact",
    network: "eip155:1",
    paymentToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    recipient: "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
  })(request);
}

export const config = {
  matcher: "/api/:path*"
};

// app/api/data/route.ts
import { NextRequest, NextResponse } from "next/server";

export async function GET(request: NextRequest) {
  return NextResponse.json({ data: "Your paid content here!" });
}
```

### Rust (Axum)

```rust
use x402_agent_sdk::adapters::axum::{x402_middleware, AxumX402Config};

let config = AxumX402Config::new(
    100,
    "v1-eip155-exact",
    "eip155:1",
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
).with_description("API Access");

// Add as Axum middleware
Router::new()
    .route("/api/data", get(data_handler))
    .layer(middleware::_fn(move |req, next| {
        x402_middleware(config.clone(), req, next)
    }));
```

---

## Payment Flow

```
┌─────────┐                    ┌─────────────┐
│  Agent  │ ───── GET /data ──▶ │   Server    │
└─────────┘                    └──────┬──────┘
                                      │
                               ┌──────▼──────┐
                               │ No Payment?  │
                               │  402 Response│
                               │ + PaymentReq │
                               └──────┬──────┘
                                      │
       ┌──────────────────────────────┘
       │ 1. Parse PaymentRequired
       │ 2. Fund Wallet
       │ 3. Sign Payment
       ▼
┌─────────────┐                    ┌─────────────┐
│   Agent    │ ──── GET /data ──▶ │   Server    │
│            │ + X-Payment-Token  │             │
└─────────────┘                    └──────┬──────┘
                                      │
                               ┌──────▼──────┐
                               │ Verify Token│
                               │  Return Data│
                               └─────────────┘
```

---

## Configuration

### Environment Variables

```bash
# Server
X402_ENABLED=true
X402_NETWORK=eip155:1
X402_PAYMENT_TOKEN=0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
X402_RECIPIENT=0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E

# Client (optional)
X402_FACILITATOR_URL=https://facilitator.x402.rs
```

### Security

The SDK includes built-in security features:

```typescript
import { InputValidator, RateLimiter, RateLimitConfig } from "./mod.ts";

// Address validation
InputValidator.validateAddress("0x742d...", "ethereum");
InputValidator.validateAddress("EPjFWdd...", "solana");
InputValidator.validateAddress("SP3K8BC...", "stacks");

// Amount validation
InputValidator.validateAmount("100");

// Rate limiting
const limiter = new RateLimiter(new RateLimitConfig({
  maxRequests: 100,
  windowSeconds: 60
}));

// Check rate limit
await limiter.check("user_address");
```

---

## Examples

### Complete Example: Express Server

```typescript
import express from "express";
import { x402Middleware, createSolanaPayment } from "./mod.ts";

const app = express();

// Require payment for all /api routes
app.use("/api", x402Middleware({
  pricePerRequest: 100,
  scheme: "v1-solana-exact",
  network: "solana:101",
  paymentToken: "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
  recipient: "YourWalletAddress",
  description: "API Access"
}));

app.get("/api/weather", (req, res) => {
  res.json({ temp: 72, condition: "sunny" });
});

app.post("/api/analyze", express.json(), (req, res) => {
  res.json({ result: "Analysis complete!" });
});

app.listen(3000, () => {
  console.log("x402 Server running on http://localhost:3000");
});
```

### Complete Example: Agent Client

```typescript
import { X402Client, createEvmPayment } from "./mod.ts";

const client = new X402Client({
  facilitatorUrl: "https://facilitator.x402.rs",
  walletPrivateKey: "your-private-key"
});

async function callPaidAPI() {
  const response = await client.request("https://api.example.com/premium-data", {
    method: "GET"
  });
  
  if (response.status === 402) {
    const paymentRequired = await response.json();
    console.log("Payment required:", paymentRequired);
    return;
  }
  
  const data = await response.json();
  console.log("Data:", data);
}

callPaidAPI();
```

---

## API Reference

### X402Server

```typescript
class X402Server {
  constructor(
    pricePerRequest: number,
    scheme: string,
    network: string,
    paymentToken: string,
    recipient: string
  );
  
  createPaymentRequired(description?: string): PaymentRequired;
  checkPayment(headers: Headers): boolean;
  middleware(): (request: Request) => Promise<Response>;
}
```

### X402Client

```typescript
class X402Client {
  constructor(config: X402ClientOptions);
  
  async request(url: string, options?: RequestInit): Promise<Response>;
}
```

### PaymentRequired

```typescript
interface PaymentRequired {
  version: number;
  scheme: string;
  network: string;
  payment_token: string;
  max_amount: string;
  recipient: string;
  expires_at?: string;
  description?: string;
}
```

---

## Support

- GitHub: https://github.com/yourorg/x402-agent-sdk
- Discord: https://discord.gg/x402
- Twitter: @x402proto
