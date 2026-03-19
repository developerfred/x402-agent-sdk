use serde::{Deserialize, Serialize};

#[cfg(feature = "stacks")]
use stacks::types::StacksAddress;

use crate::core::error::X402Error;
use crate::core::payment::PaymentRequired;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StacksNetwork {
    Mainnet,
    Testnet,
    Devnet,
}

impl StacksNetwork {
    pub fn from_chain_id(chain_id: &str) -> Result<Self, X402Error> {
        match chain_id {
            "stacks:mainnet" | "0x01" | "stacks:0x01" => Ok(StacksNetwork::Mainnet),
            "stacks:testnet" | "0x80000000" | "stacks:0x80000000" => Ok(StacksNetwork::Testnet),
            "stacks:devnet" => Ok(StacksNetwork::Devnet),
            _ => Err(X402Error::UnsupportedScheme(format!(
                "Unknown Stacks network: {}",
                chain_id
            ))),
        }
    }

    pub fn chain_id(&self) -> &str {
        match self {
            StacksNetwork::Mainnet => "0x01",
            StacksNetwork::Testnet => "0x80000000",
            StacksNetwork::Devnet => "stacks:devnet",
        }
    }

    pub fn rpc_url(&self) -> &str {
        match self {
            StacksNetwork::Mainnet => "https://stacks-node-api.mainnet.stacks.co",
            StacksNetwork::Testnet => "https://stacks-node-api.testnet.stacks.co",
            StacksNetwork::Devnet => "http://localhost:20443",
        }
    }

    pub fn explorer_url(&self) -> &str {
        match self {
            StacksNetwork::Mainnet => "https://explorer.stacks.co",
            StacksNetwork::Testnet => "https://explorer.stacks.co/?chain=testnet",
            StacksNetwork::Devnet => "http://localhost:3000",
        }
    }
}

pub const STACKS_NETWORKS: &[(&str, &str)] = &[
    ("stacks:mainnet", "0x01"),
    ("stacks:testnet", "0x80000000"),
    ("stacks:devnet", "devnet"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StacksPayment {
    pub network: StacksNetwork,
    pub recipient: String,
    pub token: StacksToken,
    pub amount: u64,
    pub signature: Option<String>,
    pub transaction_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StacksToken {
    STX,
    SIP010,
}

impl StacksToken {
    pub fn contract_id(&self, network: StacksNetwork) -> Option<String> {
        match self {
            StacksToken::STX => None,
            StacksToken::SIP010 => {
                let addr = match network {
                    StacksNetwork::Mainnet => "SP3K8BC0PPEVCV7NZ6QSRWPQ2ZQ9GFG3EEEGH1GB",
                    StacksNetwork::Testnet => "ST3K8BC0PPEVCV7NZ6QSRWPQ2ZQ9GFG3EEEGH1GB",
                    StacksNetwork::Devnet => "ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZTDN4Y76F5",
                };
                Some(format!("{}.usda-token", addr))
            }
        }
    }

    pub fn decimals(&self) -> u8 {
        match self {
            StacksToken::STX => 6,
            StacksToken::SIP010 => 6,
        }
    }

    pub fn from_token(token: &str) -> Option<Self> {
        match token.to_lowercase().as_str() {
            "stx" | "native" => Some(StacksToken::STX),
            "sip010" | "usda" => Some(StacksToken::SIP010),
            _ => None,
        }
    }
}

impl StacksPayment {
    pub fn new(network: StacksNetwork, recipient: &str, token: StacksToken, amount: u64) -> Self {
        Self {
            network,
            recipient: recipient.to_string(),
            token,
            amount,
            signature: None,
            transaction_id: None,
        }
    }

    pub fn with_signature(mut self, signature: &str) -> Self {
        self.signature = Some(signature.to_string());
        self
    }

    pub fn with_transaction_id(mut self, tx_id: &str) -> Self {
        self.transaction_id = Some(tx_id.to_string());
        self
    }

    pub fn validate_recipient(&self) -> Result<(), X402Error> {
        #[cfg(feature = "stacks")]
        {
            let addr = StacksAddress::from_string(&self.recipient)
                .map_err(|e| X402Error::ConfigError(format!("Invalid Stacks recipient: {}", e)))?;

            match self.network {
                StacksNetwork::Mainnet | StacksNetwork::Devnet => {
                    if !addr.is_mainnet() {
                        return Err(X402Error::ConfigError(
                            "Address must be mainnet for this network".to_string(),
                        ));
                    }
                }
                StacksNetwork::Testnet => {
                    if addr.is_mainnet() {
                        return Err(X402Error::ConfigError(
                            "Address must be testnet for this network".to_string(),
                        ));
                    }
                }
            }
        }

        #[cfg(not(feature = "stacks"))]
        {
            if !self.recipient.starts_with("SP") && !self.recipient.starts_with("ST") {
                return Err(X402Error::ConfigError(format!(
                    "Invalid Stacks address prefix: {}",
                    self.recipient
                )));
            }
        }

        Ok(())
    }

    pub fn to_payment_required(&self) -> PaymentRequired {
        let token = match self.token {
            StacksToken::STX => "native".to_string(),
            StacksToken::SIP010 => self.token.contract_id(self.network).unwrap_or_default(),
        };

        PaymentRequired::new(
            "v2-stacks-exact",
            &format!("stacks:{}", self.network.chain_id()),
            &token,
            &self.amount.to_string(),
            &self.recipient,
        )
    }

    pub fn from_payment_required(pr: &PaymentRequired) -> Result<Self, X402Error> {
        let network = StacksNetwork::from_chain_id(&pr.network)?;

        let token = StacksToken::from_token(&pr.payment_token).unwrap_or(StacksToken::STX);

        Ok(Self::new(
            network,
            &pr.recipient,
            token,
            pr.max_amount.parse().unwrap_or(0),
        ))
    }

    pub fn verify_transaction(&self) -> Result<bool, X402Error> {
        if let Some(tx_id) = &self.transaction_id {
            if tx_id.len() == 64 || tx_id.starts_with("0x") {
                return Ok(true);
            }
            return Err(X402Error::SignatureInvalid(format!(
                "Invalid transaction ID format: {}",
                tx_id
            )));
        }

        Err(X402Error::SignatureInvalid(
            "No transaction ID provided".to_string(),
        ))
    }
}
