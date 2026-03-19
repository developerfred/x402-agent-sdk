use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::IntoResponse,
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::core::payment::{PaymentRequired, PaymentToken};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxumX402Config {
    pub price_per_request: u64,
    pub scheme: String,
    pub network: String,
    pub payment_token: String,
    pub recipient: String,
    pub description: Option<String>,
}

impl AxumX402Config {
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
            description: None,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn create_payment_required(&self) -> PaymentRequired {
        PaymentRequired::new(
            &self.scheme,
            &self.network,
            &self.payment_token,
            &self.price_per_request.to_string(),
            &self.recipient,
        )
        .with_description(self.description.as_deref().unwrap_or("API access"))
    }
}

pub async fn x402_middleware(
    config: AxumX402Config,
    request: Request<Body>,
    next: Next,
) -> Response {
    let headers = request.headers();
    
    let payment_token = headers.get("x-payment-token");
    let payment_required_header = headers.get("x-payment-required");

    if payment_token.is_none() && payment_required_header.is_none() {
        let pr = config.create_payment_required();
        let pr_json = pr.to_header_value().unwrap_or_else(|_| "{}".to_string());

        return (
            StatusCode::PAYMENT_REQUIRED,
            [(header::HEADER_X_PAYMENT_REQUIRED, pr_json)],
            serde_json::json!({
                "error": "Payment Required",
                "payment_required": pr
            }),
        )
            .into_response();
    }

    if let Some(token) = payment_token {
        if let Ok(token_str) = token.to_str() {
            if let Ok(decoded) = PaymentToken::decode(token_str) {
                if decoded.amount.parse::<u64>().unwrap_or(0) >= config.price_per_request {
                    return next.run(request).await;
                }
            }
        }

        return (
            StatusCode::PAYMENT_REQUIRED,
            serde_json::json!({
                "error": "Insufficient payment",
                "required": config.price_per_request,
            }),
        )
            .into_response();
    }

    next.run(request).await
}

pub fn create_payment_required_response(config: &AxumX402Config) -> (StatusCode, String) {
    let pr = config.create_payment_required();
    let pr_json = pr.to_header_value().unwrap_or_else(|_| "{}".to_string());
    (StatusCode::PAYMENT_REQUIRED, pr_json)
}

pub fn check_payment(headers: &HeaderMap, required_amount: u64) -> Result<bool, String> {
    let payment_token = headers.get("x-payment-token");

    if payment_token.is_none() {
        return Ok(false);
    }

    if let Some(token) = payment_token {
        let token_str = token.to_str().map_err(|e| e.to_string())?;
        let decoded = PaymentToken::decode(token_str).map_err(|e| e.to_string())?;
        
        let amount = decoded.amount.parse::<u64>().unwrap_or(0);
        
        if amount >= required_amount {
            return Ok(true);
        }
    }

    Ok(false)
}

pub trait X402Response {
    fn x402_payment_required(self, payment_required: PaymentRequired) -> Self;
}

impl X402Response for Response<Body> {
    fn x402_payment_required(mut self, payment_required: PaymentRequired) -> Self {
        if let Ok(pr_json) = payment_required.to_header_value() {
            self.headers_mut().insert(
                header::HEADER_X_PAYMENT_REQUIRED,
                pr_json.parse().unwrap(),
            );
        }
        *self.status_mut() = StatusCode::PAYMENT_REQUIRED;
        self
    }
}

pub const HEADER_X_PAYMENT_REQUIRED: &str = "x-payment-required";
pub const HEADER_X_PAYMENT_TOKEN: &str = "x-payment-token";
