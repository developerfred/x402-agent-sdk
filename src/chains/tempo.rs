//! Tempo chain support for x402
//!
//! Tempo is a blockchain by Stripe + Paradigm designed for payments.
//! It supports the Machine Payments Protocol (MPP) with Stripe integration.

use serde::{Deserialize, Serialize};

/// Tempo network variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TempoNetwork {
    /// Tempo mainnet
    Mainnet,
    /// Tempo testnet
    Testnet,
    /// Tempo devnet
    Devnet,
}

impl TempoNetwork {
    /// Get the chain ID for this network
    pub fn chain_id(&self) -> u64 {
        match self {
            TempoNetwork::Mainnet => 5000,
            TempoNetwork::Testnet => 5001,
            TempoNetwork::Devnet => 5002,
        }
    }

    /// Get the network identifier for x402
    pub fn to_x502_network(&self) -> String {
        match self {
            TempoNetwork::Mainnet => "tempo:5000".to_string(),
            TempoNetwork::Testnet => "tempo:5001".to_string(),
            TempoNetwork::Devnet => "tempo:5002".to_string(),
        }
    }
}

impl Default for TempoNetwork {
    fn default() -> Self {
        TempoNetwork::Mainnet
    }
}

/// Supported tokens on Tempo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TempoToken {
    /// USD Coin
    USDC,
    /// USDT
    USDT,
    /// Euro Coin
    EUROC,
    /// British Pound Coin
    GBPC,
}

impl TempoToken {
    /// Get the contract address for this token on Tempo
    pub fn address(&self) -> Option<&str> {
        // Tempo uses USDC as the primary stablecoin
        // These would be the actual contract addresses on Tempo mainnet
        match self {
            TempoToken::USDC => Some("0x5b19A317c261E754bA4b3D6a3F9aB5b5c5c5c5c"), // Placeholder
            TempoToken::USDT => Some("0x6bB5d3E6E4e2b1A8d5C3E9f2D4a7c8b9e0f1a2b"), // Placeholder
            TempoToken::EUROC => Some("0x7cD4E5e3F6d5C9g0E5d4F3aE6b8d9c0f1a2b3c"), // Placeholder
            TempoToken::GBPC => Some("0x8dE5F6e4G7f6D0h1F6e5G4bAf7c9e0d2g3b4c5d"), // Placeholder
        }
    }

    /// Get the decimals for this token
    pub fn decimals(&self) -> u8 {
        match self {
            TempoToken::USDC | TempoToken::USDT => 6,
            TempoToken::EUROC | TempoToken::GBPC => 2,
        }
    }
}

/// Tempo payment structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoPayment {
    /// Network
    pub network: TempoNetwork,
    /// Recipient address
    pub recipient: String,
    /// Token to pay with
    pub token: TempoToken,
    /// Amount in smallest unit (cents for fiat-backed, lamports for crypto)
    pub amount: u64,
    /// Payment description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl TempoPayment {
    /// Create a new Tempo payment
    pub fn new(
        network: TempoNetwork,
        recipient: impl Into<String>,
        token: TempoToken,
        amount: u64,
    ) -> Self {
        Self {
            network,
            recipient: recipient.into(),
            token,
            amount,
            description: None,
        }
    }

    /// Set payment description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Convert to PaymentRequired for x402 response
    pub fn to_payment_required(&self) -> crate::core::payment::PaymentRequired {
        let token_address = self.token.address().unwrap_or("USDC");

        crate::core::payment::PaymentRequired::new(
            "v1-tempo-exact", // Tempo-specific scheme
            &self.network.to_x502_network(),
            token_address,
            &self.amount.to_string(),
            &self.recipient,
        )
    }
}

/// MPP (Machine Payments Protocol) session for Tempo
///
/// Sessions allow agents to pre-authorize spending caps
/// and stream micropayments without per-request authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoSession {
    /// Session ID
    pub session_id: String,
    /// Authorized spender address
    pub authorized_spender: String,
    /// Maximum amount authorized (in smallest unit)
    pub max_amount: u64,
    /// Amount spent so far
    pub spent: u64,
    /// Session expiry timestamp
    pub expires_at: i64,
    /// Payment token authorized
    pub token: TempoToken,
}

impl TempoSession {
    /// Check if session is still valid
    pub fn is_valid(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now < self.expires_at && self.spent < self.max_amount
    }

    /// Check if amount is within session limit
    pub fn can_spend(&self, amount: u64) -> bool {
        self.is_valid() && (self.spent + amount) <= self.max_amount
    }

    /// Record a payment
    pub fn record_payment(&mut self, amount: u64) -> Result<(), crate::X402Error> {
        if !self.can_spend(amount) {
            return Err(crate::X402Error::Payment(
                "Amount exceeds session limit or session expired".to_string(),
            ));
        }
        self.spent += amount;
        Ok(())
    }

    /// Get remaining authorized amount
    pub fn remaining(&self) -> u64 {
        self.max_amount.saturating_sub(self.spent)
    }
}

/// Stripe facilitator config for Tempo
///
/// Tempo has native Stripe integration - payments can be
/// settled in fiat via Stripe or in crypto on Tempo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoStripeConfig {
    /// Stripe publishable key
    pub stripe_publishable_key: String,
    /// Merchant ID
    pub merchant_id: String,
    /// Enable fiat settlement
    pub enable_fiat: bool,
    /// Enable crypto settlement
    pub enable_crypto: bool,
}

impl Default for TempoStripeConfig {
    fn default() -> Self {
        Self {
            stripe_publishable_key: String::new(),
            merchant_id: String::new(),
            enable_fiat: true,
            enable_crypto: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempo_network_chain_id() {
        assert_eq!(TempoNetwork::Mainnet.chain_id(), 5000);
        assert_eq!(TempoNetwork::Testnet.chain_id(), 5001);
        assert_eq!(TempoNetwork::Devnet.chain_id(), 5002);
    }

    #[test]
    fn test_tempo_payment_new() {
        let payment = TempoPayment::new(
            TempoNetwork::Mainnet,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            TempoToken::USDC,
            100,
        );

        assert_eq!(payment.network, TempoNetwork::Mainnet);
        assert_eq!(payment.amount, 100);
        assert_eq!(payment.token, TempoToken::USDC);
    }

    #[test]
    fn test_tempo_payment_to_payment_required() {
        let payment = TempoPayment::new(
            TempoNetwork::Mainnet,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            TempoToken::USDC,
            100,
        );

        let pr = payment.to_payment_required();

        assert!(pr.scheme.contains("tempo"));
        assert!(pr.network.contains("tempo"));
    }

    #[test]
    fn test_tempo_session() {
        let session = TempoSession {
            session_id: "sess_123".to_string(),
            authorized_spender: "0x742d...".to_string(),
            max_amount: 10000,
            spent: 500,
            expires_at: chrono::Utc::now().timestamp() + 3600,
            token: TempoToken::USDC,
        };

        assert!(session.is_valid());
        assert!(session.can_spend(1000));
        assert!(!session.can_spend(10000)); // Would exceed limit
    }
}
