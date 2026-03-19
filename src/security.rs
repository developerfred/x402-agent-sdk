use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
        }
    }
}

pub struct RateLimiter {
    config: RateLimitConfig,
    requests: Arc<tokio::sync::RwLock<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            requests: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn check(&self, key: &str) -> Result<(), RateLimitError> {
        let now = Instant::now();
        let window_start = now - Duration::from_secs(self.config.window_seconds);

        let mut requests = self.requests.write().await;
        
        let key_requests = requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        key_requests.retain(|&time| time > window_start);
        
        if key_requests.len() >= self.config.max_requests as usize {
            return Err(RateLimitError::Exceeded {
                max_requests: self.config.max_requests,
                window_seconds: self.config.window_seconds,
            });
        }
        
        key_requests.push(now);
        
        Ok(())
    }

    pub async fn reset(&self, key: &str) {
        let mut requests = self.requests.write().await;
        requests.remove(key);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitError {
    Exceeded { max_requests: u32, window_seconds: u64 },
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::Exceeded { max_requests, window_seconds } => {
                write!(
                    f,
                    "Rate limit exceeded: {} requests per {} seconds",
                    max_requests, window_seconds
                )
            }
        }
    }
}

impl std::error::Error for RateLimitError {}

pub struct InputValidator;

impl InputValidator {
    pub fn validate_address(address: &str, chain: &str) -> Result<(), ValidationError> {
        match chain.to_lowercase().as_str() {
            "solana" => Self::validate_solana_address(address),
            "stacks" => Self::validate_stacks_address(address),
            "ethereum" | "evm" => Self::validate_eth_address(address),
            _ => Ok(()),
        }
    }

    fn validate_solana_address(address: &str) -> Result<(), ValidationError> {
        if address.len() < 32 || address.len() > 44 {
            return Err(ValidationError::InvalidAddress(
                "Invalid Solana address length".to_string(),
            ));
        }

        let valid_chars = address.chars().all(|c| c.is_ascii_alphanumeric());
        if !valid_chars {
            return Err(ValidationError::InvalidAddress(
                "Solana address contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_stacks_address(address: &str) -> Result<(), ValidationError> {
        if !address.starts_with("SP") && !address.starts_with("ST") {
            return Err(ValidationError::InvalidAddress(
                "Stacks address must start with SP or ST".to_string(),
            ));
        }

        if address.len() < 40 || address.len() > 50 {
            return Err(ValidationError::InvalidAddress(
                "Invalid Stacks address length".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_eth_address(address: &str) -> Result<(), ValidationError> {
        if !address.starts_with("0x") {
            return Err(ValidationError::InvalidAddress(
                "Ethereum address must start with 0x".to_string(),
            ));
        }

        if address.len() != 42 {
            return Err(ValidationError::InvalidAddress(
                "Invalid Ethereum address length".to_string(),
            ));
        }

        let hex_part = &address[2..];
        let valid_hex = hex_part.chars().all(|c| c.is_ascii_hexdigit());
        if !valid_hex {
            return Err(ValidationError::InvalidAddress(
                "Ethereum address contains invalid hex characters".to_string(),
            ));
        }

        Ok(())
    }

    pub fn validate_amount(amount: &str) -> Result<u64, ValidationError> {
        let parsed = amount.parse::<u64>()
            .map_err(|_| ValidationError::InvalidAmount("Amount must be a valid integer".to_string()))?;

        if parsed == 0 {
            return Err(ValidationError::InvalidAmount("Amount must be greater than 0".to_string()));
        }

        Ok(parsed)
    }

    pub fn sanitize_header_value(value: &str) -> String {
        value
            .chars()
            .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
            .take(10000)
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    InvalidAddress(String),
    InvalidAmount(String),
    InvalidFormat(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            ValidationError::InvalidAmount(msg) => write!(f, "Invalid amount: {}", msg),
            ValidationError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}
