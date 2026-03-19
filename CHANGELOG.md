# CHANGELOG

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-03-17

### Added
- **Core Protocol**
  - PaymentRequired struct for x402 payment headers
  - PaymentToken for signed payment authorization
  - W3C Verifiable Credentials support
  - HTTP 402 response handling

- **Chain Support**
  - EVM chains: Ethereum, Polygon, Base, Arbitrum, Optimism, Avalanche, BSC, Celo, Sepolia, Amoy
  - Solana: Mainnet, Devnet, Testnet support
  - Stacks: Mainnet, Testnet, Devnet support

- **Framework Adapters**
  - Express.js middleware
  - Fastify plugin
  - Next.js middleware
  - Axum middleware (Rust)

- **Security**
  - Input validation for all chain addresses
  - Amount validation
  - Rate limiting
  - Header sanitization

- **TypeScript/Deno**
  - Full TypeScript definitions
  - Deno native module
  - WASM bindings

- **Testing**
  - 40 unit tests covering all modules
  - Core, chains, security, and agent tests

### Features
- Multi-chain payment creation
- Automatic payment token encoding/decoding
- Credential builder with expiration
- Flexible route configuration

## [0.0.1] - 2026-03-01

### Added
- Initial release
- Basic x402 protocol implementation
- Solana and EVM support prototype
