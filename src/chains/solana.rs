use serde::{Deserialize, Serialize};

#[cfg(feature = "solana")]
use solana_sdk::{pubkey::Pubkey, signature::Signature};

use crate::core::error::X402Error;
use crate::core::payment::PaymentRequired;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SolanaNetwork {
    Mainnet,
    Devnet,
    Testnet,
}

impl SolanaNetwork {
    pub fn from_chain_id(chain_id: u64) -> Result<Self, X402Error> {
        match chain_id {
            101 => Ok(SolanaNetwork::Mainnet),
            102 => Ok(SolanaNetwork::Devnet),
            103 => Ok(SolanaNetwork::Testnet),
            _ => Err(X402Error::UnsupportedScheme(format!(
                "Unknown Solana chain ID: {}",
                chain_id
            ))),
        }
    }

    pub fn chain_id(&self) -> u64 {
        match self {
            SolanaNetwork::Mainnet => 101,
            SolanaNetwork::Devnet => 102,
            SolanaNetwork::Testnet => 103,
        }
    }

    pub fn rpc_url(&self) -> &str {
        match self {
            SolanaNetwork::Mainnet => "https://api.mainnet-beta.solana.com",
            SolanaNetwork::Devnet => "https://api.devnet.solana.com",
            SolanaNetwork::Testnet => "https://api.testnet.solana.com",
        }
    }
}

pub const SOLANA_NETWORKS: &[(&str, u32)] = &[
    ("solana:mainnet", 101),
    ("solana:devnet", 102),
    ("solana:testnet", 103),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaPayment {
    pub network: SolanaNetwork,
    pub recipient: String,
    pub token: SolanaToken,
    pub amount: u64,
    pub signature: Option<String>,
    pub transaction: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SolanaToken {
    SOL,
    USDC,
    USDT,
}

impl SolanaToken {
    pub fn mint(&self) -> Option<&str> {
        match self {
            SolanaToken::SOL => None,
            SolanaToken::USDC => Some("EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ"),
            SolanaToken::USDT => Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11Mcx8eZ1kRBJ3"),
        }
    }

    pub fn from_mint(mint: &str) -> Option<Self> {
        match mint {
            "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ" => Some(SolanaToken::USDC),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11Mcx8eZ1kRBJ3" => Some(SolanaToken::USDT),
            _ => None,
        }
    }

    pub fn decimals(&self) -> u8 {
        match self {
            SolanaToken::SOL => 9,
            SolanaToken::USDC | SolanaToken::USDT => 6,
        }
    }
}

impl SolanaPayment {
    pub fn new(network: SolanaNetwork, recipient: &str, token: SolanaToken, amount: u64) -> Self {
        Self {
            network,
            recipient: recipient.to_string(),
            token,
            amount,
            signature: None,
            transaction: None,
        }
    }

    pub fn with_signature(mut self, signature: &str) -> Self {
        self.signature = Some(signature.to_string());
        self
    }

    pub fn with_transaction(mut self, transaction: &str) -> Self {
        self.transaction = Some(transaction.to_string());
        self
    }

    pub fn validate_recipient(&self) -> Result<(), X402Error> {
        #[cfg(feature = "solana")]
        {
            if let Err(_) = self.recipient.parse::<Pubkey>() {
                return Err(X402Error::ConfigError(format!(
                    "Invalid Solana recipient address: {}",
                    self.recipient
                )));
            }
        }

        #[cfg(not(feature = "solana"))]
        {
            if self.recipient.len() < 32 || self.recipient.len() > 44 {
                return Err(X402Error::ConfigError(format!(
                    "Invalid Solana recipient address length: {}",
                    self.recipient
                )));
            }
        }

        Ok(())
    }

    pub fn to_payment_required(&self) -> PaymentRequired {
        let token_address = self.token.mint().unwrap_or("native");

        PaymentRequired::new(
            "v1-solana-exact",
            &format!("solana:{}", self.network.chain_id()),
            token_address,
            &self.amount.to_string(),
            &self.recipient,
        )
    }

    pub fn from_payment_required(pr: &PaymentRequired) -> Result<Self, X402Error> {
        let network = if pr.network.contains("mainnet") {
            SolanaNetwork::Mainnet
        } else if pr.network.contains("devnet") {
            SolanaNetwork::Devnet
        } else {
            SolanaNetwork::Testnet
        };

        let token = SolanaToken::from_mint(&pr.payment_token).unwrap_or(SolanaToken::SOL);

        Ok(Self::new(
            network,
            &pr.recipient,
            token,
            pr.max_amount.parse().unwrap_or(0),
        ))
    }

    #[cfg(feature = "solana")]
    pub fn verify_signature(&self, message: &[u8]) -> Result<bool, X402Error> {
        use sha2::{Digest, Sha256};

        if let Some(sig) = &self.signature {
            let signature =
                Signature::from_str(sig).map_err(|e| X402Error::SignatureInvalid(e.to_string()))?;

            let pubkey = Pubkey::from_str(&self.recipient)
                .map_err(|e| X402Error::ConfigError(e.to_string()))?;

            let mut hasher = Sha256::new();
            hasher.update(message);
            let hash = hasher.finalize();

            Ok(pubkey.verify(&hash, &signature))
        } else {
            Err(X402Error::SignatureInvalid(
                "No signature provided".to_string(),
            ))
        }
    }

    #[cfg(not(feature = "solana"))]
    pub fn verify_signature(&self, _message: &[u8]) -> Result<bool, X402Error> {
        Err(X402Error::ConfigError(
            "Solana feature not enabled".to_string(),
        ))
    }
}
