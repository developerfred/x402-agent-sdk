# Integration Guide

This guide walks through integrating x402 payments into your API.

## Quick Start

### 1. Choose Your Stack

| Language | Framework | Adapter |
|----------|-----------|---------|
| Rust | Axum | `x402_agent_sdk::adapters::axum` |
| TypeScript | Express | `@x402/sdk/adapters/express` |
| TypeScript | Fastify | `@x402/sdk/adapters/fastify` |
| TypeScript | Next.js | `@x402/sdk/adapters/nextjs` |
| Python | FastAPI | `x402_fastapi` |
| Python | Flask | `x402_flask` |
| Python | Django | `x402_django` |
| Go | Gin | `github.com/x402/adapters/go/gin` |
| Go | Fiber | `github.com/x402/adapters/go/fiber` |
| Java | Spring Boot | `com.x402` |
| C# | .NET | `X402Middleware` |
| Ruby | Rails | `x402_rails` |
| PHP | Laravel | `X402Middleware` |

### 2. Basic Integration

#### Server Side (Receiving Payments)

**Express.js Example:**

```typescript
import express from 'express';
import { x402Middleware } from '@x402/sdk/adapters/express';

const app = express();

app.use(x402Middleware({
  pricePerRequest: 100,           // $1.00
  scheme: 'v1-eip155-exact',      // x402 scheme
  network: 'eip155:8453',         // Base network
  paymentToken: '0x833589fCD6eDb6E08f4c7C32D4f71B54bdA02913', // USDC on Base
  recipient: '0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E',
}));

app.get('/api/premium', (req, res) => {
  res.json({ 
    data: 'This is premium content!',
    message: 'Paid with x402' 
  });
});

app.listen(3000);
```

**FastAPI Example:**

```python
from fastapi import FastAPI
from x402_fastapi import X402Middleware, X402Config

app = FastAPI()

app.add_middleware(
    X402Middleware,
    config=X402Config(
        price_per_request=100,
        network="eip155:8453",
        payment_token="0x833589fCD6eDb6E08f4c7C32D4f71B54bdA02913",
        recipient="0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E"
    )
)

@app.get("/api/premium")
async def premium_content():
    return {"data": "Premium content"}
```

#### Client Side (Making Payments)

**TypeScript:**

```typescript
import { X402Client } from '@x402/sdk/client';

const client = new X402Client({
  wallet: myWallet,
  facilitatorUrl: 'https://facilitator.x402.rs',
});

const response = await client.request('https://api.example.com/premium', {
  method: 'GET',
});
```

### 3. Payment Flow

```
┌─────────┐    GET /api/premium    ┌────────────┐
│  Client │ ─────────────────────▶ │   Server   │
└─────────┘                        └─────┬──────┘
                                          │
                                  ┌────────▼────────┐
                                  │ No Payment?    │
                                  │ 402 Response   │
                                  │ + PaymentReq   │
                                  └─────┬──────────┘
                                         │
       ┌─────────────────────────────────┘
       │ 1. Parse X-Payment-Required header
       │ 2. Fund wallet with USDC
       │ 3. Sign payment transaction
       ▼
┌─────────┐  GET + X-Payment-Token  ┌────────────┐
│  Client │ ────────────────────────▶ │   Server   │
└─────────┘                          └─────┬──────┘
                                            │
                                    ┌────────▼────────┐
                                    │ Verify Payment │
                                    │ Return Data    │
                                    └────────────────┘
```

### 4. Testing Locally

#### Start a Local Blockchain

```bash
# Using Hardhat for EVM
npx hardhat node

# Using Solana
solana-test-validator
```

#### Run Your Server

```bash
# Point to local network
export X402_NETWORK="eip155:31337"  # Hardhat
export X402_RECIPIENT="0xYourAddress"
```

### 5. Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `X402_ENABLED` | Enable/disable x402 | `true` |
| `X402_NETWORK` | Default network | `eip155:1` |
| `X402_PAYMENT_TOKEN` | Default token | USDC address |
| `X402_RECIPIENT` | Payment recipient | - |
| `X402_PRICE_PER_REQUEST` | Default price | 100 |
| `X402_FACILITATOR_URL` | Payment facilitator | `https://facilitator.x402.rs` |

## Advanced Usage

### Custom Payment Requirements

```typescript
app.use(x402Middleware((req) => {
  // Dynamic pricing based on endpoint
  const prices: Record<string, number> = {
    '/api/read': 10,
    '/api/write': 50,
    '/api/admin': 500,
  };
  
  const path = req.path;
  const price = prices[path] || 100;
  
  return {
    pricePerRequest: price,
    scheme: 'v1-eip155-exact',
    network: 'eip155:8453',
    paymentToken: '0x833589fCD6eDb6E08f4c7C32D4f71B54bdA02913',
    recipient: process.env.RECIPIENT_ADDRESS,
  };
}));
```

### Tempo MPP Sessions

For AI agents that need to make many requests:

```typescript
import { TempoSessionManager } from '@x402/sdk/adapters/tempo';

const manager = new TempoSessionManager();

// Create session with $1000 cap
const session = manager.createSession(
  agentWalletAddress,
  1000000n,  // $1000 in cents
  'USDC',
  86400      // 24 hours
);

// Agent can now make requests within budget
async function makePaidRequest(endpoint: string) {
  const sessionId = session.sessionId;
  
  const response = await fetch(endpoint, {
    headers: {
      'X-MPP-Session': sessionId,
    }
  });
  
  if (response.status === 402) {
    // Payment required - handle error
  }
  
  return response.json();
}
```

### Multi-Chain Support

```typescript
// Accept payments on multiple chains
app.use(x402Middleware((req) => {
  const chain = req.query.chain || 'base';
  
  const chains: Record<string, { network: string, token: string }> = {
    base: { network: 'eip155:8453', token: '0x833589fCD6eDb6E08f4c7C32D4f71B54bdA02913' },
    polygon: { network: 'eip155:137', token: '0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174' },
    solana: { network: 'solana:101', token: 'EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ' },
    tempo: { network: 'tempo:5000', token: 'USDC' },  // Stripe integration!
  };
  
  return {
    pricePerRequest: 100,
    scheme: 'v1-eip155-exact',
    ...chains[chain as string],
  };
}));
```

## Troubleshooting

### Payments Not Working

1. **Check network**: Ensure client and server use the same network
2. **Verify token**: Ensure the token address is correct for the network
3. **Check balance**: Ensure client has sufficient funds
4. **Facilitator**: Verify facilitator URL is accessible

### 402 Response Missing Headers

Ensure middleware is registered before your routes:

```typescript
// ❌ Wrong - middleware after routes
app.get('/api', handler);
app.use(x402Middleware(config));

// ✅ Correct - middleware before routes  
app.use(x402Middleware(config));
app.get('/api', handler);
```

### Session Expired

```typescript
try {
  const result = await client.request(url);
} catch (error) {
  if (error.code === 'SESSION_EXPIRED') {
    // Create new session
    const newSession = manager.createSession(...);
    // Retry request with new session
  }
}
```

## Next Steps

- [API Reference](./API_REFERENCE.md)
- [Supported Blockchains](./BLOCKCHAINS.md)
- [Security Guide](./SECURITY.md)
- [Examples](../examples/)
