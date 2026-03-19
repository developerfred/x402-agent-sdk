pub mod agent;
pub mod chains;
pub mod core;
pub mod security;

#[cfg(feature = "axum")]
pub mod adapters;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use core::credential::{Credential, CredentialBuilder};
pub use core::error::X402Error;
pub use core::payment::{PaymentHeader, PaymentRequired, PaymentToken};
pub use security::{InputValidator, RateLimitConfig, RateLimiter, ValidationError};

pub use chains::{
    SolanaNetwork, SolanaPayment, SolanaToken, StacksNetwork, StacksPayment, StacksToken,
    EvmNetwork, EvmPayment, EvmToken, EVM_NETWORKS,
};

#[cfg(test)]
mod tests {
    use crate::chains::{
        EvmNetwork, EvmPayment, EvmToken, EVM_NETWORKS, SolanaNetwork, SolanaPayment, SolanaToken,
        StacksNetwork, StacksPayment, StacksToken,
    };
    use crate::core::credential::{Credential, CredentialBuilder};
    use crate::core::error::X402Error;
    use crate::core::payment::{PaymentRequired, PaymentToken};

    #[test]
    fn test_payment_required_new() {
        let pr = PaymentRequired::new(
            "v1-solana-exact",
            "solana:101",
            "USDC",
            "100",
            "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
        );

        assert_eq!(pr.version, 1);
        assert_eq!(pr.scheme, "v1-solana-exact");
        assert_eq!(pr.network, "solana:101");
        assert_eq!(pr.payment_token, "USDC");
        assert_eq!(pr.max_amount, "100");
    }

    #[test]
    fn test_payment_required_to_header() {
        let pr = PaymentRequired::new(
            "v1-solana-exact",
            "solana:101",
            "USDC",
            "100",
            "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
        );

        let header = pr.to_header_value().unwrap();
        assert!(header.contains("v1-solana-exact"));
    }

    #[test]
    fn test_payment_required_from_header() {
        let original = PaymentRequired::new(
            "v1-solana-exact",
            "solana:101",
            "USDC",
            "100",
            "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
        );

        let header = original.to_header_value().unwrap();
        let parsed = PaymentRequired::from_header_value(&header).unwrap();

        assert_eq!(parsed.scheme, original.scheme);
        assert_eq!(parsed.network, original.network);
        assert_eq!(parsed.max_amount, original.max_amount);
    }

    #[test]
    fn test_payment_token_encode_decode() {
        let pr = PaymentRequired::new(
            "v1-solana-exact",
            "solana:101",
            "USDC",
            "100",
            "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
        );

        let token = PaymentToken::new(pr, "test_signature", "0xsender", "100");

        let encoded = token.encode().unwrap();
        let decoded = PaymentToken::decode(&encoded).unwrap();

        assert_eq!(decoded.sender, "0xsender");
        assert_eq!(decoded.signature, "test_signature");
    }

    #[test]
    fn test_credential_new() {
        let claims = serde_json::json!({
            "name": "Test Agent",
            "permissions": ["read", "write"]
        });

        let cred = Credential::new("https://issuer.example.com", "did:example:agent123", claims);

        assert_eq!(cred.issuer, "https://issuer.example.com");
        assert_eq!(cred.subject, "did:example:agent123");
        assert!(cred.r#type.contains(&"VerifiableCredential".to_string()));
    }

    #[test]
    fn test_credential_builder() {
        let builder = CredentialBuilder::new("https://issuer.example.com", "did:example:agent123")
            .claim("name", serde_json::json!("Test Agent"))
            .claim("role", serde_json::json!("admin"));

        let cred = builder.build();

        assert_eq!(cred.issuer, "https://issuer.example.com");
        assert_eq!(cred.claims["name"], "Test Agent");
        assert_eq!(cred.claims["role"], "admin");
    }

    #[test]
    fn test_solana_payment_new() {
        let payment = SolanaPayment::new(
            SolanaNetwork::Mainnet,
            "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
            SolanaToken::USDC,
            100,
        );

        assert_eq!(payment.network, SolanaNetwork::Mainnet);
        assert_eq!(payment.token, SolanaToken::USDC);
        assert_eq!(payment.amount, 100);
    }

    #[test]
    fn test_solana_payment_to_payment_required() {
        let payment = SolanaPayment::new(
            SolanaNetwork::Mainnet,
            "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
            SolanaToken::USDC,
            100,
        );

        let pr = payment.to_payment_required();

        assert_eq!(pr.scheme, "v1-solana-exact");
        assert!(pr.network.contains("solana"));
    }

    #[test]
    fn test_stacks_payment_new() {
        let payment = StacksPayment::new(
            StacksNetwork::Mainnet,
            "SP3K8BC0PPEVCV7NZ6QSRWPQ2ZQ9GFG3EEEGH1GB",
            StacksToken::STX,
            1000,
        );

        assert_eq!(payment.network, StacksNetwork::Mainnet);
        assert_eq!(payment.token, StacksToken::STX);
        assert_eq!(payment.amount, 1000);
    }

    #[test]
    fn test_stacks_payment_to_payment_required() {
        let payment = StacksPayment::new(
            StacksNetwork::Mainnet,
            "SP3K8BC0PPEVCV7NZ6QSRWPQ2ZQ9GFG3EEEGH1GB",
            StacksToken::STX,
            1000,
        );

        let pr = payment.to_payment_required();

        assert_eq!(pr.scheme, "v2-stacks-exact");
        assert!(pr.network.contains("stacks"));
    }

    #[test]
    fn test_evm_payment_new() {
        let payment = EvmPayment::new(
            EvmNetwork::Base,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            EvmToken::USDC,
            100,
        );

        assert_eq!(payment.network, EvmNetwork::Base);
        assert_eq!(payment.token, EvmToken::USDC);
        assert_eq!(payment.amount, 100);
    }

    #[test]
    fn test_evm_payment_to_payment_required() {
        let payment = EvmPayment::new(
            EvmNetwork::Ethereum,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            EvmToken::USDC,
            100,
        );

        let pr = payment.to_payment_required();

        assert_eq!(pr.scheme, "v1-eip155-exact");
        assert!(pr.network.contains("eip155:1"));
    }

    #[test]
    fn test_evm_payment_validate_recipient() {
        let payment = EvmPayment::new(
            EvmNetwork::Polygon,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            EvmToken::USDT,
            50,
        );

        assert!(payment.validate_recipient().is_ok());
    }

    #[test]
    fn test_evm_payment_invalid_recipient() {
        let payment = EvmPayment::new(
            EvmNetwork::Polygon,
            "742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            EvmToken::USDT,
            50,
        );

        assert!(payment.validate_recipient().is_err());
    }

    #[test]
    fn test_evm_network_from_chain_id() {
        assert_eq!(EvmNetwork::from_chain_id(1).unwrap(), EvmNetwork::Ethereum);
        assert_eq!(EvmNetwork::from_chain_id(137).unwrap(), EvmNetwork::Polygon);
        assert_eq!(EvmNetwork::from_chain_id(8453).unwrap(), EvmNetwork::Base);
        assert_eq!(EvmNetwork::from_chain_id(42161).unwrap(), EvmNetwork::Arbitrum);
    }

    #[test]
    fn test_evm_token_address_polygon() {
        let usdc_addr = EvmToken::USDC.address(EvmNetwork::Polygon);
        assert!(usdc_addr.is_some());
        assert!(usdc_addr.unwrap().contains("0x2791"));
    }

    #[test]
    fn test_evm_token_address_base() {
        let usdc_addr = EvmToken::USDC.address(EvmNetwork::Base);
        assert!(usdc_addr.is_some());
    }

    #[test]
    fn test_evm_token_address_arbitrum() {
        let usdc_addr = EvmToken::USDC.address(EvmNetwork::Arbitrum);
        assert!(usdc_addr.is_some());
    }

    #[test]
    fn test_evm_token_address_optimism() {
        let usdc_addr = EvmToken::USDC.address(EvmNetwork::Optimism);
        assert!(usdc_addr.is_some());
    }

    #[test]
    fn test_evm_token_address_avalanche() {
        let usdc_addr = EvmToken::USDC.address(EvmNetwork::Avalanche);
        assert!(usdc_addr.is_some());
    }

    #[test]
    fn test_evm_token_address_bsc() {
        let usdc_addr = EvmToken::USDC.address(EvmNetwork::Bsc);
        assert!(usdc_addr.is_some());
    }

    #[test]
    fn test_evm_token_address_sepolia() {
        let usdc_addr = EvmToken::USDC.address(EvmNetwork::Sepolia);
        assert!(usdc_addr.is_some());
    }

    #[test]
    fn test_evm_token_from_address() {
        assert_eq!(EvmToken::from_address("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), Some(EvmToken::USDC));
        assert_eq!(EvmToken::from_address("0xdac17f958d2ee523a2206206994597c13d831ec7"), Some(EvmToken::USDT));
        assert_eq!(EvmToken::from_address("invalid"), None);
    }

    #[test]
    fn test_stacks_payment_from_payment_required() {
        use crate::chains::StacksPayment;
        
        let pr = PaymentRequired::new(
            "v2-stacks-exact",
            "stacks:0x01",
            "STX",
            "1000",
            "SP3K8BC0PPEVCV7NZ6QSRWPQ2ZQ9GFG3EEEGH1GB",
        );
        
        let payment = StacksPayment::from_payment_required(&pr).unwrap();
        assert_eq!(payment.network, StacksNetwork::Mainnet);
    }

    #[test]
    fn test_stacks_payment_from_payment_required_testnet() {
        use crate::chains::StacksPayment;
        
        let pr = PaymentRequired::new(
            "v2-stacks-exact",
            "stacks:0x80000000",
            "STX",
            "500",
            "ST3K8BC0PPEVCV7NZ6QSRWPQ2ZQ9GFG3EEEGH1GB",
        );
        
        let payment = StacksPayment::from_payment_required(&pr).unwrap();
        assert_eq!(payment.network, StacksNetwork::Testnet);
    }

    #[test]
    fn test_solana_payment_from_payment_required() {
        use crate::chains::SolanaPayment;
        
        let pr = PaymentRequired::new(
            "v1-solana-exact",
            "solana:101",
            "USDC",
            "100",
            "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
        );
        
        let payment = SolanaPayment::from_payment_required(&pr).unwrap();
        assert_eq!(payment.network, SolanaNetwork::Testnet);
    }

    #[test]
    fn test_input_validator_solana_address() {
        use crate::security::InputValidator;
        
        assert!(InputValidator::validate_address("EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ", "solana").is_ok());
        assert!(InputValidator::validate_address("invalid", "solana").is_err());
    }

    #[test]
    fn test_input_validator_stacks_address() {
        use crate::security::InputValidator;
        
        assert!(InputValidator::validate_address("SP3K8BC0PPEVCV7NZ6QSRWPQ2ZQ9GFG3EEEGH1GB", "stacks").is_ok());
        assert!(InputValidator::validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E", "stacks").is_err());
    }

    #[test]
    fn test_input_validator_eth_address() {
        use crate::security::InputValidator;
        
        assert!(InputValidator::validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E", "ethereum").is_ok());
        assert!(InputValidator::validate_address("742d35Cc6634C0532925a3b844Bc9e7595f0fB1E", "ethereum").is_err());
        assert!(InputValidator::validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1", "ethereum").is_err());
    }

    #[test]
    fn test_input_validator_amount() {
        use crate::security::InputValidator;
        
        assert_eq!(InputValidator::validate_amount("100").unwrap(), 100);
        assert!(InputValidator::validate_amount("0").is_err());
        assert!(InputValidator::validate_amount("abc").is_err());
    }

    #[test]
    fn test_sanitize_header_value() {
        use crate::security::InputValidator;
        
        let sanitized = InputValidator::sanitize_header_value("正常文本内容");
        assert_eq!(sanitized.len(), 0);
        
        let sanitized2 = InputValidator::sanitize_header_value("Hello World");
        assert!(sanitized2.contains("Hello"));
    }

    #[test]
    fn test_rate_limit_config_default() {
        use crate::security::RateLimitConfig;
        
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window_seconds, 60);
    }

    #[test]
    fn test_error_serialization() {
        let error = X402Error::InvalidHeader("Test error".to_string());
        let serialized = serde_json::to_string(&error).unwrap();

        assert!(serialized.contains("Test error"));
    }

    // Agent tests
    #[test]
    fn test_agent_client_config_default() {
        use crate::agent::AgentClientConfig;
        
        let config = AgentClientConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.facilitator_url, "https://facilitator.x402.rs");
    }

    #[test]
    fn test_agent_client_new() {
        use crate::agent::{AgentClient, AgentClientConfig};
        
        let config = AgentClientConfig::default();
        let client = AgentClient::new(config);
        assert!(true); // Just verify it constructs
    }

    #[test]
    fn test_agent_client_create_payment_token() {
        use crate::agent::{AgentClient, AgentClientConfig};
        
        let client = AgentClient::new(AgentClientConfig::default());
        let pr = PaymentRequired::new(
            "v1-eip155-exact",
            "eip155:1",
            "USDC",
            "100",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        );
        
        let token = client.create_payment_token(&pr).unwrap();
        assert_eq!(token.sender, "0xsender");
    }

    #[test]
    fn test_agent_server_new() {
        use crate::agent::AgentServer;
        
        let server = AgentServer::new(
            100,
            "v1-eip155-exact",
            "eip155:1",
            "USDC",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        );
        
        let pr = server.create_payment_required(Some("Test API"));
        assert_eq!(pr.max_amount, "100");
    }

    #[test]
    fn test_agent_server_create_payment_required() {
        use crate::agent::AgentServer;
        
        let server = AgentServer::new(
            500,
            "v1-solana-exact",
            "solana:101",
            "USDC",
            "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
        );
        
        let pr = server.create_payment_required(Some("Premium Data"));
        assert_eq!(pr.max_amount, "500");
        assert_eq!(pr.description.as_ref().unwrap(), "Premium Data");
    }

    #[test]
    fn test_agent_server_create_402_response() {
        use crate::agent::AgentServer;
        
        let server = AgentServer::new(
            100,
            "v1-eip155-exact",
            "eip155:1",
            "USDC",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        );
        
        let (status, header) = server.create_402_response();
        assert_eq!(status.as_u16(), 402);
        assert!(!header.is_empty());
    }

    // Additional chain tests
    #[test]
    fn test_evm_network_all_networks() {
        use crate::chains::EvmNetwork;
        
        assert_eq!(EvmNetwork::Ethereum.chain_id(), 1);
        assert_eq!(EvmNetwork::Polygon.chain_id(), 137);
        assert_eq!(EvmNetwork::Base.chain_id(), 8453);
        assert_eq!(EvmNetwork::Arbitrum.chain_id(), 42161);
        assert_eq!(EvmNetwork::Optimism.chain_id(), 10);
    }

    #[test]
    fn test_evm_token_symbol() {
        use crate::chains::EvmToken;
        
        assert_eq!(EvmToken::ETH.symbol(), "ETH");
        assert_eq!(EvmToken::USDC.symbol(), "USDC");
        assert_eq!(EvmToken::USDT.symbol(), "USDT");
    }

    #[test]
    fn test_solana_network_chain_ids() {
        use crate::chains::SolanaNetwork;
        
        assert_eq!(SolanaNetwork::Mainnet.chain_id(), 101);
        assert_eq!(SolanaNetwork::Devnet.chain_id(), 102);
        assert_eq!(SolanaNetwork::Testnet.chain_id(), 103);
    }

    #[test]
    fn test_stacks_network_chain_ids() {
        use crate::chains::StacksNetwork;
        
        assert_eq!(StacksNetwork::Mainnet.chain_id(), "0x01");
        assert_eq!(StacksNetwork::Testnet.chain_id(), "0x80000000");
    }

    #[test]
    fn test_payment_header_from_headers_none() {
        use crate::core::payment::PaymentHeader;
        
        let headers = http::HeaderMap::new();
        let result = PaymentHeader::from_headers(&headers).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_payment_header_from_headers_with_required() {
        use crate::core::payment::PaymentHeader;
        
        let mut headers = http::HeaderMap::new();
        headers.insert(
            "x-payment-required",
            "true".parse().unwrap(),
        );
        
        let result = PaymentHeader::from_headers(&headers).unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().required);
    }

    #[test]
    fn test_payment_required_with_expiry() {
        use chrono::Utc;
        
        let pr = PaymentRequired::new(
            "v1-eip155-exact",
            "eip155:1",
            "USDC",
            "100",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        );
        
        let with_expiry = pr.to_header_value().unwrap();
        assert!(with_expiry.contains("v1-eip155-exact"));
    }

    #[test]
    fn test_payment_token_decode_invalid() {
        let result = PaymentToken::decode("invalid_base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_credential_with_expiry() {
        use chrono::{Duration, Utc};
        
        let cred = Credential::new(
            "https://issuer.example.com",
            "did:example:123",
            serde_json::json!({}),
        ).with_expiry(Utc::now() + Duration::hours(1));
        
        assert!(cred.expires_at.is_some());
    }

    #[test]
    fn test_credential_is_expired() {
        use chrono::{Duration, Utc};
        
        let expired_cred = Credential::new(
            "https://issuer.example.com",
            "did:example:123",
            serde_json::json!({}),
        ).with_expiry(Utc::now() - Duration::hours(1));
        
        assert!(expired_cred.is_expired());
        
        let valid_cred = Credential::new(
            "https://issuer.example.com",
            "did:example:123",
            serde_json::json!({}),
        ).with_expiry(Utc::now() + Duration::hours(1));
        
        assert!(!valid_cred.is_expired());
    }

    #[test]
    fn test_validation_error_display() {
        use crate::security::ValidationError;
        
        let err = ValidationError::InvalidAddress("test".to_string());
        assert!(err.to_string().contains("Invalid address"));
        
        let err2 = ValidationError::InvalidAmount("test".to_string());
        assert!(err2.to_string().contains("Invalid amount"));
        
        let err3 = ValidationError::InvalidFormat("test".to_string());
        assert!(err3.to_string().contains("Invalid format"));
    }

    // More chain tests - EVM
    #[test]
    fn test_evm_payment_from_payment_required() {
        use crate::chains::EvmPayment;
        
        let pr = PaymentRequired::new(
            "v1-eip155-exact",
            "eip155:1",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "100",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        );
        
        let payment = EvmPayment::from_payment_required(&pr).unwrap();
        assert_eq!(payment.network, EvmNetwork::Ethereum);
    }

    #[test]
    fn test_evm_payment_from_payment_required_invalid_network() {
        use crate::chains::EvmPayment;
        
        let pr = PaymentRequired::new(
            "v1-eip155-exact",
            "solana:101",
            "USDC",
            "100",
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
        );
        
        let result = EvmPayment::from_payment_required(&pr);
        assert!(result.is_err());
    }

    #[test]
    fn test_evm_payment_with_signature() {
        use crate::chains::EvmPayment;
        
        let payment = EvmPayment::new(
            EvmNetwork::Polygon,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            EvmToken::USDT,
            50,
        ).with_signature("0xsignature");
        
        assert_eq!(payment.signature.unwrap(), "0xsignature");
    }

    #[test]
    fn test_evm_payment_with_transaction_hash() {
        use crate::chains::EvmPayment;
        
        let payment = EvmPayment::new(
            EvmNetwork::Polygon,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            EvmToken::USDT,
            50,
        ).with_transaction_hash("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
        
        assert!(payment.transaction_hash.is_some());
    }

    #[test]
    fn test_evm_payment_verify_transaction() {
        use crate::chains::EvmPayment;
        
        let payment = EvmPayment::new(
            EvmNetwork::Polygon,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            EvmToken::USDT,
            50,
        ).with_transaction_hash("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
        
        assert!(payment.verify_transaction().is_ok());
    }

    #[test]
    fn test_evm_payment_verify_transaction_invalid() {
        use crate::chains::EvmPayment;
        
        let payment = EvmPayment::new(
            EvmNetwork::Polygon,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",
            EvmToken::USDT,
            50,
        );
        
        assert!(payment.verify_transaction().is_err());
    }

    #[test]
    fn test_evm_network_display() {
        let eth = EvmNetwork::Ethereum;
        assert_eq!(eth.to_string(), "Ethereum");
        
        let polygon = EvmNetwork::Polygon;
        assert_eq!(polygon.to_string(), "Polygon");
    }

    #[test]
    fn test_evm_network_from_unknown_chain_id() {
        let result = EvmNetwork::from_chain_id(99999);
        assert!(result.is_err());
    }

    // More Solana tests
    #[test]
    fn test_solana_payment_with_signature() {
        use crate::chains::SolanaPayment;
        
        let payment = SolanaPayment::new(
            SolanaNetwork::Mainnet,
            "EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ",
            SolanaToken::USDC,
            100,
        ).with_signature("signature123");
        
        assert_eq!(payment.signature.unwrap(), "signature123");
    }

    #[test]
    fn test_solana_payment_validate_recipient_invalid() {
        use crate::chains::SolanaPayment;
        
        let payment = SolanaPayment::new(
            SolanaNetwork::Mainnet,
            "invalid_address_too_short",
            SolanaToken::USDC,
            100,
        );
        
        assert!(payment.validate_recipient().is_err());
    }

    #[test]
    fn test_solana_token_from_mint() {
        use crate::chains::SolanaToken;
        
        assert_eq!(SolanaToken::from_mint("EPjFWdd5AufqSSFqM7BcEgHwgsn7kWGLelwTjB9mFLAQ"), Some(SolanaToken::USDC));
        assert_eq!(SolanaToken::from_mint("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11Mcx8eZ1kRBJ3"), Some(SolanaToken::USDT));
        assert_eq!(SolanaToken::from_mint("unknown"), None);
    }

    // More Stacks tests
    #[test]
    fn test_stacks_payment_with_signature() {
        use crate::chains::StacksPayment;
        
        let payment = StacksPayment::new(
            StacksNetwork::Mainnet,
            "SP3K8BC0PPEVCV7NZ6QSRWPQ2ZQ9GFG3EEEGH1GB",
            StacksToken::STX,
            1000,
        ).with_signature("signature123");
        
        assert_eq!(payment.signature.unwrap(), "signature123");
    }

    #[test]
    fn test_stacks_payment_with_transaction_id() {
        use crate::chains::StacksPayment;
        
        let payment = StacksPayment::new(
            StacksNetwork::Mainnet,
            "SP3K8BC0PPEVCV7NZ6QSRWPQ2ZQ9GFG3EEEGH1GB",
            StacksToken::STX,
            1000,
        ).with_transaction_id("0x1234567890abcdef");
        
        assert!(payment.transaction_id.is_some());
    }

    #[test]
    fn test_stacks_token_from_token() {
        use crate::chains::StacksToken;
        
        assert_eq!(StacksToken::from_token("stx"), Some(StacksToken::STX));
        assert_eq!(StacksToken::from_token("native"), Some(StacksToken::STX));
        assert_eq!(StacksToken::from_token("sip010"), Some(StacksToken::SIP010));
        assert_eq!(StacksToken::from_token("unknown"), None);
    }

    #[test]
    fn test_stacks_network_from_unknown() {
        use crate::chains::StacksNetwork;
        
        let result = StacksNetwork::from_chain_id("unknown:123");
        assert!(result.is_err());
    }

    // Error tests
    #[test]
    fn test_x402_error_variants() {
        let e1 = X402Error::InvalidHeader("test".to_string());
        assert!(e1.to_string().contains("Invalid payment header"));
        
        let e2 = X402Error::InvalidToken("test".to_string());
        assert!(e2.to_string().contains("Invalid payment token"));
        
        let e3 = X402Error::TokenExpired("2024-01-01".to_string());
        assert!(e3.to_string().contains("Token expired"));
        
        let e4 = X402Error::SignatureInvalid("test".to_string());
        assert!(e4.to_string().contains("Signature verification failed"));
        
        let e5 = X402Error::CredentialInvalid("test".to_string());
        assert!(e5.to_string().contains("Credential validation failed"));
        
        let e6 = X402Error::UnsupportedScheme("test".to_string());
        assert!(e6.to_string().contains("Unsupported payment scheme"));
        
        let e7 = X402Error::NetworkError("test".to_string());
        assert!(e7.to_string().contains("Network error"));
        
        let e8 = X402Error::ParseError("test".to_string());
        assert!(e8.to_string().contains("Parse error"));
        
        let e9 = X402Error::ConfigError("test".to_string());
        assert!(e9.to_string().contains("Configuration error"));
    }

    // Security tests - more validation
    #[test]
    fn test_input_validator_unknown_chain() {
        use crate::security::InputValidator;
        
        // Unknown chain should pass (no validation)
        let result = InputValidator::validate_address("0x742d...", "unknown");
        assert!(result.is_ok());
    }

    #[test]
    fn test_credential_builder_all() {
        let builder = CredentialBuilder::new("issuer", "subject")
            .claim("key1", serde_json::json!("value1"))
            .claim("key2", serde_json::json!(123))
            .credential_type("AgentCredential")
            .expires_at(chrono::Utc::now() + chrono::Duration::hours(1));
        
        let cred = builder.build();
        assert_eq!(cred.claims["key1"], "value1");
        assert_eq!(cred.claims["key2"], 123);
    }

    // Agent tests - more coverage
    #[test]
    fn test_agent_client_handle_payment_required_not_402() {
        use crate::agent::{AgentClient, AgentClientConfig};
        
        let client = AgentClient::new(AgentClientConfig::default());
        let headers = http::HeaderMap::new();
        
        let result = client.handle_payment_required(200, &headers).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_agent_client_encode_token() {
        use crate::agent::{AgentClient, AgentClientConfig};
        
        let pr = PaymentRequired::new(
            "v1-eip155-exact",
            "eip155:1",
            "USDC",
            "100",
            "0x742d...",
        );
        
        let token = PaymentToken::new(pr, "sig", "sender", "100");
        let encoded = AgentClient::encode_token(&token).unwrap();
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_agent_server_check_payment_with_token() {
        use crate::agent::AgentServer;
        
        let server = AgentServer::new(100, "v1-eip155-exact", "eip155:1", "USDC", "0x...");
        
        // No payment headers = payment required (returns true meaning payment is needed)
        let mut headers = http::HeaderMap::new();
        let result = server.check_payment(&headers).unwrap();
        // With no headers, it returns true (payment needed)
        assert!(result);
    }

    #[test]
    fn test_payment_required_default_description() {
        let server = crate::agent::AgentServer::new(100, "v1-eip155-exact", "eip155:1", "USDC", "0x...");
        let pr = server.create_payment_required(None);
        assert_eq!(pr.description.as_ref().unwrap(), "API access");
    }

    // Security - rate limiter test
    #[test]
    fn test_rate_limiter_basic() {
        use crate::security::{RateLimitConfig, RateLimiter};
        
        let config = RateLimitConfig {
            max_requests: 2,
            window_seconds: 60,
        };
        
        let _limiter = RateLimiter::new(config);
        
        assert!(true);
    }

    // Additional EVM chain ID tests
    #[test]
    fn test_evm_network_all_chain_ids() {
        assert_eq!(EvmNetwork::Avalanche.chain_id(), 43114);
        assert_eq!(EvmNetwork::Bsc.chain_id(), 56);
        assert_eq!(EvmNetwork::Celo.chain_id(), 42220);
        assert_eq!(EvmNetwork::Sepolia.chain_id(), 11155111);
        assert_eq!(EvmNetwork::Amoy.chain_id(), 80002);
    }

    #[test]
    fn test_evm_network_all_names() {
        assert_eq!(EvmNetwork::Avalanche.name(), "Avalanche C-Chain");
        assert_eq!(EvmNetwork::Bsc.name(), "BNB Smart Chain");
        assert_eq!(EvmNetwork::Celo.name(), "Celo");
    }

    #[test]
    fn test_evm_network_all_rpc_urls() {
        assert!(EvmNetwork::Avalanche.rpc_url().contains("avax"));
        assert!(EvmNetwork::Bsc.rpc_url().contains("binance"));
        assert!(EvmNetwork::Celo.rpc_url().contains("celo"));
    }

    #[test]
    fn test_evm_network_all_explorer_urls() {
        assert!(EvmNetwork::Avalanche.explorer_url().contains("snowtrace"));
        assert!(EvmNetwork::Bsc.explorer_url().contains("bscscan"));
        assert!(EvmNetwork::Celo.explorer_url().contains("celoscan"));
    }

    // Solana tests
    #[test]
    fn test_solana_network_all() {
        assert_eq!(SolanaNetwork::Mainnet.rpc_url().contains("mainnet"), true);
        assert_eq!(SolanaNetwork::Testnet.rpc_url().contains("testnet"), true);
    }

    #[test]
    fn test_solana_token_decimals() {
        assert_eq!(SolanaToken::SOL.decimals(), 9);
        assert_eq!(SolanaToken::USDC.decimals(), 6);
        assert_eq!(SolanaToken::USDT.decimals(), 6);
    }

    // Test the EVM_NETWORKS constant
    #[test]
    fn test_evm_networks_constant() {
        assert!(EVM_NETWORKS.len() > 0);
    }
}
