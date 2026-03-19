use std::sync::Arc;
use rocket::request::{self, Request, FromRequest, Outcome};
use rocket::response::{self, Response, Responder};
use rocket::http::Status;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct X402Config {
    pub price_per_request: u64,
    pub scheme: String,
    pub network: String,
    pub payment_token: String,
    pub recipient: String,
    pub description: String,
    pub expiry_seconds: u64,
    pub facilitator_url: String,
}

impl Default for X402Config {
    fn default() -> Self {
        Self {
            price_per_request: 100,
            scheme: "v1-eip155-exact".to_string(),
            network: "eip155:1".to_string(),
            payment_token: String::new(),
            recipient: String::new(),
            description: "API access".to_string(),
            expiry_seconds: 3600,
            facilitator_url: "https://facilitator.x402.rs".to_string(),
        }
    }
}

impl X402Config {
    pub fn new(
        price_per_request: u64,
        network: &str,
        payment_token: &str,
        recipient: &str,
    ) -> Self {
        Self {
            price_per_request,
            network: network.to_string(),
            payment_token: payment_token.to_string(),
            recipient: recipient.to_string(),
            ..Default::default()
        }
    }

    pub fn create_payment_required(&self, path: &str) -> PaymentRequired {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let expiry = now + self.expiry_seconds;
        
        let salt = format!("{:x}", simple_hash(&format!("{}:{}", path, now)));
        
        let body = serde_json::json!({
            "network": self.network,
            "payment_token": self.payment_token,
            "amount": self.price_per_request.to_string(),
            "recipient": self.recipient,
            "valid_after": now,
            "salt": salt,
        });

        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        PaymentRequired {
            version: 1,
            scheme: self.scheme.clone(),
            network: self.network.clone(),
            payment_token: self.payment_token.clone(),
            max_amount: self.price_per_request.to_string(),
            recipient: self.recipient.clone(),
            description: self.description.clone(),
            expiry,
            salt,
            instructions: vec![Instruction {
                protocol: "https".to_string(),
                method: "POST".to_string(),
                url: format!("{}/v1/payment", self.facilitator_url),
                headers,
                body,
            }],
        }
    }
}

fn simple_hash(input: &str) -> u128 {
    let mut hash: u128 = 0;
    for (i, byte) in input.bytes().enumerate() {
        hash = hash.wrapping_add((byte as u128).wrapping_mul((i as u128).wrapping_add(1)));
    }
    hash
}

#[derive(Serialize, Deserialize)]
pub struct PaymentRequired {
    pub version: u8,
    pub scheme: String,
    pub network: String,
    #[serde(rename = "payment_token")]
    pub payment_token: String,
    #[serde(rename = "max_amount")]
    pub max_amount: String,
    pub recipient: String,
    pub description: String,
    pub expiry: u64,
    pub salt: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Serialize, Deserialize)]
pub struct Instruction {
    pub protocol: String,
    pub method: String,
    pub url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: serde_json::Value,
}

impl<'r> Responder<'r> for PaymentRequired {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'r> {
        Response::build()
            .status(Status::PaymentRequired)
            .sized_body(None, std::io::Cursor::new(serde_json::to_string(&self).unwrap()))
            .ok()
    }
}

pub struct X402Guard {
    pub config: Arc<X402Config>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for X402Guard {
    type Error = ();
    
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let config = req.guard::<&rocket::State<Arc<X402Config>>>().await?;
        
        let token = req.headers()
            .get_one("X-Payment-Token");

        if let Some(token) = token {
            if is_valid_token(token) {
                return Outcome::Success(X402Guard { config: config.inner().clone() });
            }
        }

        let path = req.uri().path().to_string();
        let payment_required = config.create_payment_required(&path);
        
        Outcome::Error((Status::PaymentRequired, ()))
    }
}

fn is_valid_token(token: &str) -> bool {
    use base64::Engine;
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(token) {
        let s = String::from_utf8_lossy(&decoded);
        return s.contains("\"signature\"") && s.contains("\"sender\"");
    }
    false
}

pub fn require_payment() -> X402Config {
    X402Config::default()
}

pub fn require_payment_with_config(config: X402Config) -> X402Config {
    config
}
