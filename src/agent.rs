use serde::{Deserialize, Serialize};

use crate::core::error::X402Error;
use crate::core::payment::{PaymentHeader, PaymentRequired, PaymentToken};

use hex;
use k256::ecdsa::signature::Signer;
use k256::ecdsa::{Signature as EcdsaSignature, SigningKey as EcdsaSigningKey};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::elliptic_curve::FieldBytes;
use k256::SecretKey;
use sha3::{Digest, Keccak256};

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
        // Get private key from config
        let private_key = self.config.wallet_private_key.as_ref().ok_or_else(|| {
            X402Error::InvalidHeader("No wallet private key configured".to_string())
        })?;

        // Parse private key (supports hex or 0x prefix)
        let key_bytes = hex::decode(private_key.trim_start_matches("0x"))
            .map_err(|e| X402Error::InvalidHeader(format!("Invalid private key: {}", e)))?;

        let secret_key = SecretKey::from_bytes(FieldBytes::<k256::Secp256k1>::from_slice(
            &key_bytes,
        ))
        .map_err(|e| X402Error::InvalidHeader(format!("Invalid private key format: {}", e)))?;

        // Derive sender address from key
        let sender = compute_address(&secret_key);

        // Create signature payload
        let payload = create_signature_payload(payment_required);

        // Sign the payload
        let signing_key = EcdsaSigningKey::from(secret_key);
        let signature: EcdsaSignature = signing_key.sign(&payload);
        let signature_hex = format!("0x{}", hex::encode(signature.to_bytes()));

        Ok(PaymentToken::new(
            payment_required.clone(),
            &signature_hex,
            &sender,
            &payment_required.max_amount,
        ))
    }

    pub fn encode_token(token: &PaymentToken) -> Result<String, X402Error> {
        token.encode()
    }
}

fn compute_address(secret_key: &SecretKey) -> String {
    let public_key = secret_key.public_key();
    let uncompressed = public_key.to_encoded_point(false);
    let hash = Keccak256::digest(&uncompressed.as_bytes()[1..]);
    let address = &hash.as_slice()[12..];
    format!("0x{}", hex::encode(address))
}

fn create_signature_payload(payment_required: &PaymentRequired) -> Vec<u8> {
    let mut hasher = Keccak256::new();

    hasher.update(payment_required.scheme.as_bytes());
    hasher.update(payment_required.network.as_bytes());
    hasher.update(payment_required.payment_token.as_bytes());
    hasher.update(payment_required.max_amount.as_bytes());
    hasher.update(payment_required.recipient.as_bytes());

    if let Some(desc) = &payment_required.description {
        hasher.update(desc.as_bytes());
    }

    if let Some(exp) = &payment_required.expires_at {
        hasher.update(exp.timestamp().to_string().as_bytes());
    }

    hasher.finalize().to_vec()
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
