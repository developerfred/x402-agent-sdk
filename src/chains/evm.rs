use serde::{Deserialize, Serialize};

use crate::core::error::X402Error;
use crate::core::payment::PaymentRequired;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvmNetwork {
    Ethereum,
    Polygon,
    Arbitrum,
    Optimism,
    Base,
    Avalanche,
    Bsc,
    Celo,
    Sepolia,
    Amoy,
}

impl EvmNetwork {
    pub fn from_chain_id(chain_id: u64) -> Result<Self, X402Error> {
        match chain_id {
            1 => Ok(EvmNetwork::Ethereum),
            137 => Ok(EvmNetwork::Polygon),
            42161 => Ok(EvmNetwork::Arbitrum),
            10 => Ok(EvmNetwork::Optimism),
            8453 => Ok(EvmNetwork::Base),
            43114 => Ok(EvmNetwork::Avalanche),
            56 => Ok(EvmNetwork::Bsc),
            42220 => Ok(EvmNetwork::Celo),
            11155111 => Ok(EvmNetwork::Sepolia),
            80002 => Ok(EvmNetwork::Amoy),
            _ => Err(X402Error::UnsupportedScheme(format!(
                "Unknown EVM chain ID: {}",
                chain_id
            ))),
        }
    }

    pub fn chain_id(&self) -> u64 {
        match self {
            EvmNetwork::Ethereum => 1,
            EvmNetwork::Polygon => 137,
            EvmNetwork::Arbitrum => 42161,
            EvmNetwork::Optimism => 10,
            EvmNetwork::Base => 8453,
            EvmNetwork::Avalanche => 43114,
            EvmNetwork::Bsc => 56,
            EvmNetwork::Celo => 42220,
            EvmNetwork::Sepolia => 11155111,
            EvmNetwork::Amoy => 80002,
        }
    }

    pub fn rpc_url(&self) -> &str {
        match self {
            EvmNetwork::Ethereum => "https://eth.llamarpc.com",
            EvmNetwork::Polygon => "https://polygon-rpc.com",
            EvmNetwork::Arbitrum => "https://arb1.arbitrum.io/rpc",
            EvmNetwork::Optimism => "https://mainnet.optimism.io",
            EvmNetwork::Base => "https://mainnet.base.org",
            EvmNetwork::Avalanche => "https://api.avax.network/ext/bc/C/rpc",
            EvmNetwork::Bsc => "https://bsc-dataseed.binance.org",
            EvmNetwork::Celo => "https://forno.celo.org",
            EvmNetwork::Sepolia => "https://rpc.sepolia.org",
            EvmNetwork::Amoy => "https://rpc-amoy.polygon.technology",
        }
    }

    pub fn explorer_url(&self) -> &str {
        match self {
            EvmNetwork::Ethereum => "https://etherscan.io",
            EvmNetwork::Polygon => "https://polygonscan.com",
            EvmNetwork::Arbitrum => "https://arbiscan.io",
            EvmNetwork::Optimism => "https://optimistic.etherscan.io",
            EvmNetwork::Base => "https://basescan.org",
            EvmNetwork::Avalanche => "https://snowtrace.io",
            EvmNetwork::Bsc => "https://bscscan.com",
            EvmNetwork::Celo => "https://celoscan.io",
            EvmNetwork::Sepolia => "https://sepolia.etherscan.io",
            EvmNetwork::Amoy => "https://amoy.polygonscan.com",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            EvmNetwork::Ethereum => "Ethereum",
            EvmNetwork::Polygon => "Polygon",
            EvmNetwork::Arbitrum => "Arbitrum One",
            EvmNetwork::Optimism => "Optimism",
            EvmNetwork::Base => "Base",
            EvmNetwork::Avalanche => "Avalanche C-Chain",
            EvmNetwork::Bsc => "BNB Smart Chain",
            EvmNetwork::Celo => "Celo",
            EvmNetwork::Sepolia => "Sepolia (Testnet)",
            EvmNetwork::Amoy => "Polygon Amoy (Testnet)",
        }
    }
}

impl std::fmt::Display for EvmNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

pub const EVM_NETWORKS: &[(&str, u64, &str)] = &[
    ("ethereum", 1, "Ethereum"),
    ("polygon", 137, "Polygon"),
    ("arbitrum", 42161, "Arbitrum One"),
    ("optimism", 10, "Optimism"),
    ("base", 8453, "Base"),
    ("avalanche", 43114, "Avalanche C-Chain"),
    ("bsc", 56, "BNB Smart Chain"),
    ("celo", 42220, "Celo"),
    ("sepolia", 11155111, "Sepolia (Testnet)"),
    ("amoy", 80002, "Polygon Amoy (Testnet)"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmPayment {
    pub network: EvmNetwork,
    pub recipient: String,
    pub token: EvmToken,
    pub amount: u64,
    pub signature: Option<String>,
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvmToken {
    ETH,
    WETH,
    USDC,
    USDT,
    DAI,
    WBTC,
}

impl EvmToken {
    pub fn address(&self, network: EvmNetwork) -> Option<String> {
        let addr = match self {
            EvmToken::ETH => return None,
            EvmToken::WETH => match network {
                EvmNetwork::Ethereum => "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
                EvmNetwork::Polygon => "0x7ceB23fD6bC0adD59E62add25504582C41297ba0",
                EvmNetwork::Base => "0x4200000000000000000000000000000000000006",
                EvmNetwork::Arbitrum => "0x82af49447d8a07e3bd95bd0d56f35241523fbab1",
                EvmNetwork::Optimism => "0x4200000000000000000000000000000000000006",
                EvmNetwork::Avalanche => "0x53f7c5869a589C72b152e49f7Be43f7E2908D692",
                EvmNetwork::Bsc => "0x2170Ed0880ac9A755fd29B2688956BD959F933F8",
                _ => return None,
            },
            EvmToken::USDC => match network {
                EvmNetwork::Ethereum => "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                EvmNetwork::Polygon => "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
                EvmNetwork::Base => "0x833589fCD6eDb6E08f4c7C32D4f71B54bdA02913",
                EvmNetwork::Arbitrum => "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
                EvmNetwork::Optimism => "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85",
                EvmNetwork::Avalanche => "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E",
                EvmNetwork::Bsc => "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d",
                EvmNetwork::Celo => "0x37fE750F3b8F21D47D79Fc37DA12C4A5A3BfAf80",
                EvmNetwork::Sepolia => "0x1c7D4B196Cb0C7B01d743F51a9bF63Bd1C5bF85A",
                EvmNetwork::Amoy => "0x41E94Eb019C0762f9Bfcf9Fb1E5876Bf7339e4B4",
            },
            EvmToken::USDT => match network {
                EvmNetwork::Ethereum => "0xdAC17F958D2ee523a2206206994597C13D831ec7",
                EvmNetwork::Polygon => "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",
                EvmNetwork::Base => "0xfde4C96c859187D6301C4d2bA1B2F2d7a0e8B26",
                EvmNetwork::Arbitrum => "0xFd086b7Cd5C8cD3D6a202CbCe81862b3B0d3b9Ad",
                EvmNetwork::Optimism => "0x94b008aA00579c1307B0EF2c49487cb9aEcF7526",
                EvmNetwork::Avalanche => "0x9702230A8Ea53601f5cD2dc4f74723f3233C80b9",
                EvmNetwork::Bsc => "0x55d398326f99059fF775485246999027B3197955",
                EvmNetwork::Celo => "0x4988CC2bC6528CaF6A5dA7fFfE5D0b6e2d3c9A1F",
                EvmNetwork::Sepolia => "0xA02fF9Ea3D5ED30f5C94EB4D3Cf42e81C8fC0b1d",
                EvmNetwork::Amoy => "0x1D9fC72DdDd4B83D3F1B6a65e47b1d3c9A3c3F9D1",
            },
            EvmToken::DAI => match network {
                EvmNetwork::Ethereum => "0x6B175474E89094C44Da98b954EedeAC495271d0F",
                EvmNetwork::Polygon => "0x53f7c5869a589C72b152e49f7Be43f7E2908D692",
                EvmNetwork::Base => "0x4eDC8c15f5B5C9A3cC27E0c5c8C3bB5d6c5bD1e",
                _ => return None,
            },
            EvmToken::WBTC => match network {
                EvmNetwork::Ethereum => "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
                EvmNetwork::Polygon => "0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6",
                EvmNetwork::Base => "0x2Ae3F1Ec7F1F5012CFEab0185bfC7DA3cE0f4A6",
                _ => return None,
            },
        };
        Some(addr.to_string())
    }

    pub fn decimals(&self) -> u8 {
        match self {
            EvmToken::ETH | EvmToken::WETH => 18,
            EvmToken::USDC | EvmToken::USDT | EvmToken::DAI => 6,
            EvmToken::WBTC => 8,
        }
    }

    pub fn from_address(address: &str) -> Option<Self> {
        let lower = address.to_lowercase();
        match lower.as_str() {
            "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2" => Some(EvmToken::WETH),
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48" => Some(EvmToken::USDC),
            "0xdac17f958d2ee523a2206206994597c13d831ec7" => Some(EvmToken::USDT),
            "0x6b175474e89094c44da98b954eedeac495271d0f" => Some(EvmToken::DAI),
            "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599" => Some(EvmToken::WBTC),
            _ => None,
        }
    }

    pub fn symbol(&self) -> &str {
        match self {
            EvmToken::ETH => "ETH",
            EvmToken::WETH => "WETH",
            EvmToken::USDC => "USDC",
            EvmToken::USDT => "USDT",
            EvmToken::DAI => "DAI",
            EvmToken::WBTC => "WBTC",
        }
    }
}

impl EvmPayment {
    pub fn new(network: EvmNetwork, recipient: &str, token: EvmToken, amount: u64) -> Self {
        Self {
            network,
            recipient: recipient.to_string(),
            token,
            amount,
            signature: None,
            transaction_hash: None,
        }
    }

    pub fn with_signature(mut self, signature: &str) -> Self {
        self.signature = Some(signature.to_string());
        self
    }

    pub fn with_transaction_hash(mut self, tx_hash: &str) -> Self {
        self.transaction_hash = Some(tx_hash.to_string());
        self
    }

    pub fn validate_recipient(&self) -> Result<(), X402Error> {
        if !self.recipient.starts_with("0x") {
            return Err(X402Error::ConfigError(
                "EVM recipient must start with 0x".to_string(),
            ));
        }

        if self.recipient.len() != 42 {
            return Err(X402Error::ConfigError(format!(
                "Invalid EVM address length: {}",
                self.recipient.len()
            )));
        }

        let hex_part = &self.recipient[2..];
        let valid_hex = hex_part.chars().all(|c| c.is_ascii_hexdigit());
        if !valid_hex {
            return Err(X402Error::ConfigError(
                "EVM address contains invalid hex characters".to_string(),
            ));
        }

        Ok(())
    }

    pub fn to_payment_required(&self) -> PaymentRequired {
        let token_address = self
            .token
            .address(self.network)
            .unwrap_or_else(|| "native".to_string());

        PaymentRequired::new(
            "v1-eip155-exact",
            &format!("eip155:{}", self.network.chain_id()),
            &token_address,
            &self.amount.to_string(),
            &self.recipient,
        )
    }

    pub fn from_payment_required(pr: &PaymentRequired) -> Result<Self, X402Error> {
        if !pr.network.starts_with("eip155:") {
            return Err(X402Error::UnsupportedScheme(
                "Not an EVM network".to_string(),
            ));
        }

        let chain_id: u64 = pr
            .network
            .trim_start_matches("eip155:")
            .parse()
            .map_err(|_| X402Error::ParseError("Invalid chain ID".to_string()))?;

        let network = EvmNetwork::from_chain_id(chain_id)?;

        let token = if pr.payment_token == "native" || pr.payment_token.is_empty() {
            EvmToken::ETH
        } else {
            EvmToken::from_address(&pr.payment_token).unwrap_or(EvmToken::USDC)
        };

        Ok(Self::new(
            network,
            &pr.recipient,
            token,
            pr.max_amount.parse().unwrap_or(0),
        ))
    }

    pub fn verify_transaction(&self) -> Result<bool, X402Error> {
        if let Some(tx_hash) = &self.transaction_hash {
            if tx_hash.starts_with("0x") && tx_hash.len() == 66 {
                return Ok(true);
            }
            return Err(X402Error::SignatureInvalid(format!(
                "Invalid transaction hash format: {}",
                tx_hash
            )));
        }

        Err(X402Error::SignatureInvalid(
            "No transaction hash provided".to_string(),
        ))
    }
}

pub type EvmChainId = u64;

pub const ETH_MAINNET: EvmChainId = 1;
pub const POLYGON_MAINNET: EvmChainId = 137;
pub const BASE_MAINNET: EvmChainId = 8453;
pub const ARBITRUM_MAINNET: EvmChainId = 42161;
pub const OPTIMISM_MAINNET: EvmChainId = 10;
pub const AVALANCHE_MAINNET: EvmChainId = 43114;
pub const BSC_MAINNET: EvmChainId = 56;
pub const CELO_MAINNET: EvmChainId = 42220;
pub const SEPOLIA_TESTNET: EvmChainId = 11155111;
pub const AMOY_TESTNET: EvmChainId = 80002;
