# Security Guide

This document outlines security best practices for using x402-agent-sdk.

## Overview

The x402-agent-sdk implements multiple security layers:

- **Input Validation**: Sanitize all user inputs
- **Address Validation**: Verify wallet addresses are valid
- **Rate Limiting**: Prevent abuse and DoS attacks
- **Signature Verification**: Ensure payment authenticity
- **Token Expiration**: Prevent replay attacks

## Input Validation

### Address Validation

Always validate payment addresses before use:

```rust
use x402_agent_sdk::InputValidator;

// Validate Ethereum address
InputValidator::validate_address(
    "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
    "ethereum"
)?;

// Validate Solana address
InputValidator::validate_address(
    "Gk7sh8NBJ2sVEzYWqGDhJqY7J2Z3F4g5h6i7j8k9l0",
    "solana"
)?;
```

### Amount Validation

```rust
use x402_agent_sdk::InputValidator;

// Validate payment amount
InputValidator::validate_amount("100")?;  // ✅
InputValidator::validate_amount("0")?;     // ❌ Error - zero amount
InputValidator::validate_amount("-10")?;   // ❌ Error - negative
InputValidator::validate_amount("abc")?;   // ❌ Error - not a number
```

### Network Validation

```rust
InputValidator::validate_network("eip155:1")?;      // ✅ Ethereum
InputValidator::validate_network("eip155:8453")?;    // ✅ Base
InputValidator::validate_network("solana:101")?;     // ✅ Solana
InputValidator::validate_network("invalid")?;        // ❌ Error
```

## Rate Limiting

Protect your API from abuse with built-in rate limiting:

```rust
use x402_agent_sdk::{RateLimiter, RateLimitConfig};

let limiter = RateLimiter::new(RateLimitConfig {
    max_requests: 100,      // Max requests per window
    window_seconds: 60,    // Time window in seconds
});

// Check rate limit before processing
limiter.check(&client_address).await?;
```

### Per-User Rate Limits

```rust
use x402_agent_sdk::{RateLimiter, RateLimitConfig};

let limiter = RateLimiter::new(RateLimitConfig {
    max_requests: 10,
    window_seconds: 60,
});

// Different limits for different users
limiter.check(&premium_user).await?;   // Higher priority
limiter.check(&free_user).await?;      // Lower priority
```

## Payment Verification

### Server-Side Verification

Always verify payments server-side:

```rust
use x402_agent_sdk::PaymentToken;

async fn verify_payment(token: &str) -> Result<PaymentToken, X402Error> {
    let payment = PaymentToken::decode(token)?;
    
    // Verify on-chain
    verify_on_chain(&payment).await?;
    
    // Check expiration
    if payment.is_expired() {
        return Err(X402Error::TokenExpired(...));
    }
    
    Ok(payment)
}
```

### Signature Verification

```rust
use x402_agent_sdk::EvmPayment;

let payment = EvmPayment::new(...);

// Verify signature
payment.verify_signature(&signature, &message)?;
```

## Network Security

### Use HTTPS

Always use HTTPS in production:

```typescript
// ❌ Wrong - HTTP
app.listen(3000);

// ✅ Correct - HTTPS
import https from 'https';
import fs from 'fs';

const options = {
  key: fs.readFileSync('key.pem'),
  cert: fs.readFileSync('cert.pem'),
};

https.createServer(options, app).listen(3000);
```

### Environment Variables

Never commit secrets:

```bash
# ✅ Correct - Environment variables
export X402_RECIPIENT="0x742d..."
export STRIPE_SECRET="sk_live_..."

# ❌ Wrong - Hardcoded secrets
const recipient = "0x742d..."; // Don't do this!
```

## Best Practices

### 1. Use Established Networks

```rust
// ✅ Recommended - Known networks
EvmNetwork::Ethereum,
EvmNetwork::Base,
SolanaNetwork::Mainnet,

// ⚠️ Caution - Verify custom network
EvmNetwork::from_chain_id(12345),
```

### 2. Set Reasonable Expiration

```rust
let pr = PaymentRequired::new(...)
    .with_expiry(chrono::Utc::now() + chrono::Duration::minutes(5));
```

### 3. Log Security Events

```rust
use tracing::{info, warn, error};

if let Err(e) = limiter.check(&address).await {
    warn!("Rate limit exceeded for {}", address);
    // Alert security team
}
```

### 4. Implement Circuit Breaker

```rust
// Temporarily disable payments if fraud detected
if fraud_detected {
    disable_payments();
    // Alert and investigate
}
```

## Compliance

### Data Privacy

- Don't store payment tokens unnecessarily
- Clear sensitive data after verification
- Comply with GDPR, CCPA requirements

### Anti-Money Laundering (AML)

- Implement KYC for high-value transactions
- Monitor for suspicious patterns
- Report large transactions as required

## Security Checklist

Before production:

- [ ] HTTPS enabled
- [ ] Rate limiting configured
- [ ] Input validation on all endpoints
- [ ] Payment verification implemented
- [ ] Secrets in environment variables
- [ ] Logging and monitoring active
- [ ] SSL/TLS certificates valid
- [ ] Dependencies updated
- [ ] Security audit completed

## Reporting Security Issues

If you find a security vulnerability:

1. **Don't** open a public issue
2. **Email** security@example.com
3. **Include** detailed reproduction steps
4. **Wait** for acknowledgment before disclosure

We appreciate responsible disclosure!
