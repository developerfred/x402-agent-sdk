use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::X402Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequired {
    pub version: u32,
    pub scheme: String,
    pub network: String,
    pub payment_token: String,
    pub max_amount: String,
    pub recipient: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
}

impl PaymentRequired {
    pub fn new(
        scheme: &str,
        network: &str,
        payment_token: &str,
        max_amount: &str,
        recipient: &str,
    ) -> Self {
        Self {
            version: 1,
            scheme: scheme.to_string(),
            network: network.to_string(),
            payment_token: payment_token.to_string(),
            max_amount: max_amount.to_string(),
            recipient: recipient.to_string(),
            expires_at: None,
            description: None,
        }
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn to_header_value(&self) -> Result<String, X402Error> {
        serde_json::to_string(self).map_err(|e| X402Error::ParseError(e.to_string()))
    }

    pub fn from_header_value(value: &str) -> Result<Self, X402Error> {
        serde_json::from_str(value).map_err(|e| X402Error::ParseError(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentToken {
    pub id: String,
    pub payment_required: PaymentRequired,
    pub signature: String,
    pub sender: String,
    pub amount: String,
    pub nonce: u64,
    pub created_at: DateTime<Utc>,
}

impl PaymentToken {
    pub fn new(
        payment_required: PaymentRequired,
        signature: &str,
        sender: &str,
        amount: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            payment_required,
            signature: signature.to_string(),
            sender: sender.to_string(),
            amount: amount.to_string(),
            nonce: Utc::now().timestamp() as u64,
            created_at: Utc::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = &self.payment_required.expires_at {
            Utc::now() > *expires_at
        } else {
            false
        }
    }

    pub fn encode(&self) -> Result<String, X402Error> {
        let json = serde_json::to_string(self).map_err(|e| X402Error::ParseError(e.to_string()))?;
        Ok(BASE64.encode(json.as_bytes()))
    }

    pub fn decode(encoded: &str) -> Result<Self, X402Error> {
        let bytes = BASE64
            .decode(encoded)
            .map_err(|e| X402Error::ParseError(e.to_string()))?;
        let json = String::from_utf8(bytes).map_err(|e| X402Error::ParseError(e.to_string()))?;
        serde_json::from_str(&json).map_err(|e| X402Error::ParseError(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentHeader {
    pub required: bool,
    pub payment_required: Option<PaymentRequired>,
}

impl PaymentHeader {
    pub fn from_headers(headers: &http::HeaderMap) -> Result<Option<Self>, X402Error> {
        if let Some(header) = headers.get("X-Payment-Required") {
            let value = header
                .to_str()
                .map_err(|e| X402Error::InvalidHeader(e.to_string()))?;

            if value == "true" || value == "1" {
                return Ok(Some(PaymentHeader {
                    required: true,
                    payment_required: None,
                }));
            }

            let pr = PaymentRequired::from_header_value(value)?;
            return Ok(Some(PaymentHeader {
                required: true,
                payment_required: Some(pr),
            }));
        }

        if let Some(header) = headers.get("X-Payment-Token") {
            let _token = header
                .to_str()
                .map_err(|e| X402Error::InvalidToken(e.to_string()))?;
        }

        Ok(None)
    }

    pub fn create_402_response(&self) -> (http::StatusCode, Option<String>) {
        if self.required {
            if let Some(pr) = &self.payment_required {
                let header_value = pr.to_header_value().unwrap_or_default();
                return (http::StatusCode::PAYMENT_REQUIRED, Some(header_value));
            }
            return (http::StatusCode::PAYMENT_REQUIRED, None);
        }
        (http::StatusCode::OK, None)
    }
}

pub const HEADER_PAYMENT_REQUIRED: &str = "X-Payment-Required";
pub const HEADER_PAYMENT_TOKEN: &str = "X-Payment-Token";
pub const HEADER_X_402_VERSION: &str = "X-402-Version";
