# x402 Agent SDK - Architecture Design

## Project Overview
**Name:** x402-agent-sdk  
**Goal:** Transform any API into an x402 payment-enabled API with TypeScript/Deno bindings

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    x402-agent-sdk                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐  │
│  │   Core Rust    │  │  WASM Bindings  │  │  Deno Mod   │  │
│  │   Library      │  │  (wasm-bindgen) │  │  (TypeScript│  │
│  └────────┬────────┘  └────────┬────────┘  └──────┬──────┘  │
│           │                    │                   │         │
│           └──────────┬─────────┴───────────────────┘         │
│                      │                                       │
│              ┌───────▼───────┐                               │
│              │  x402-rs     │  (reuse existing types)       │
│              │  Foundation   │                               │
│              └───────────────┘                               │
└─────────────────────────────────────────────────────────────┘
```

## Core Modules

### 1. x402-core (Rust)
- **Payment Header Parsing**: Parse `X-Payment-Required`, `X-Payment-Token` headers
- **Credential Generation**: W3C Verifiable Credentials for agent authentication
- **Request Validation**: Validate payment requirements, signatures, expiry
- **Error Handling**: x402-specific error types

### 2. x402-wasm (WASM Bindings)
- **wasm-bindgen** bindings for all core functions
- **JavaScript/TypeScript** compatible API
- **Deno** native support

### 3. x402-agent (High-Level API)
- **Middleware**: Easy integration with existing APIs
- **Client**: Automatic payment handling for agents
- **Server**: Quick setup for payment-enabled endpoints

## Key Features

### Payment Flow
1. Client requests protected endpoint
2. Server returns 402 + PaymentRequired header
3. Client parses payment info, funds wallet
4. Client retries with payment token
5. Server verifies and processes

### W3C Verifiable Credentials
- Generate credentials for agent identity
- Support multiple VC formats (JWT, JSON-LD)
- Integration with existing credential frameworks

## Dependencies

### Rust
- x402-types (from x402-rs)
- serde/serde_json (serialization)
- base64 (token encoding)
- chrono (timestamps)
- thiserror (error handling)
- wasm-bindgen (WASM supportScript/D)

### Typeeno
- Native Deno module
- Type definitions
- Client utilities

## File Structure

```
x402-agent-sdk/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── payment.rs        # Payment header parsing
│   │   ├── credential.rs     # W3C VC generation
│   │   ├── validation.rs    # Request validation
│   │   └── error.rs         # Error types
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── client.rs        # Agent client
│   │   └── server.rs        # Server utilities
│   └── wasm/
│       └── lib.rs           # WASM bindings
├── deno/                     # Deno module
│   ├── mod.ts
│   ├── types.ts
│   └── examples/
├── tests/
│   └── integration/
└── examples/
    └── basic/
```

## Security Considerations

- Input validation on all payment headers
- Signature verification for credentials
- Rate limiting support
- No secret key exposure in logs
- Audit-ready transaction logging
