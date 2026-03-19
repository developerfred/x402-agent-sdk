use serde::{Deserialize, Serialize};

use crate::core::error::X402Error;
use crate::core::payment::{PaymentHeader, PaymentRequired, PaymentToken};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentClientConfig {
    pub wallet_private_key: Option<String>,
    pub facilitator_url: String,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

impl Default for AgentClientConfig {
    fn default() -> Self {
        Self {
            wallet_private_key: None,
            facilitator_url: "https://facilitator.x402.rs".to_string(),
            max_retries: 3,
            timeout_seconds: 30,
        }
    }
}

pub struct AgentClient {
    config: AgentClientConfig,
}

impl AgentClient {
    pub fn new(config: AgentClientConfig) -> Self {
        Self { config }
    }

    pub fn handle_payment_required(
        &self,
        status: u16,
        headers: &http::HeaderMap,
    ) -> Result<Option<PaymentToken>, X402Error> {
        if status != 402 {
            return Ok(None);
        }

        let payment_header = PaymentHeader::from_headers(headers)?;

        if let Some(header) = payment_header {
            if let Some(pr) = header.payment_required {
                return Err(X402Error::InvalidHeader(format!(
                    "Payment required: {} {}",
                    pr.scheme, pr.max_amount
                )));
            }
        }

        Ok(None)
    }

    pub fn create_payment_token(
        &self,
        payment_required: &PaymentRequired,
    ) -> Result<PaymentToken, X402Error> {
        let signature = "placeholder_signature";

        Ok(PaymentToken::new(
            payment_required.clone(),
            signature,
            "0xsender",
            &payment_required.max_amount,
        ))
    }

    pub fn encode_token(token: &PaymentToken) -> Result<String, X402Error> {
        token.encode()
    }
}

pub struct AgentServer {
    price_per_request: u64,
    scheme: String,
    network: String,
    payment_token: String,
    recipient: String,
}

impl AgentServer {
    pub fn new(
        price_per_request: u64,
        scheme: &str,
        network: &str,
        payment_token: &str,
        recipient: &str,
    ) -> Self {
        Self {
            price_per_request,
            scheme: scheme.to_string(),
            network: network.to_string(),
            payment_token: payment_token.to_string(),
            recipient: recipient.to_string(),
        }
    }

    pub fn create_payment_required(&self, description: Option<&str>) -> PaymentRequired {
        PaymentRequired::new(
            &self.scheme,
            &self.network,
            &self.payment_token,
            &self.price_per_request.to_string(),
            &self.recipient,
        )
        .with_description(description.unwrap_or("API access"))
    }

    pub fn check_payment(&self, headers: &http::HeaderMap) -> Result<bool, X402Error> {
        let payment_header = PaymentHeader::from_headers(headers)?;

        if let Some(header) = payment_header {
            if header.required {
                if header.payment_required.is_some() {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    pub fn create_402_response(&self) -> (http::StatusCode, String) {
        let pr = self.create_payment_required(None);
        let header_value = pr.to_header_value().unwrap_or_default();
        (http::StatusCode::PAYMENT_REQUIRED, header_value)
    }
}
