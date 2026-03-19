use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage, HttpResponse, http::header::HeaderValue};
use futures_util::future::ok;
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::TimeStamp;

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
        
        let salt = format!("{:x}", md5_hash(&format!("{}:{}", path, now)));
        
        let body = json!({
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

fn md5_hash(input: &str) -> u128 {
    let mut hash: u128 = 0;
    for (i, byte) in input.bytes().enumerate() {
        hash = hash.wrapping_add((byte as u128).wrapping_mul((i as u128).wrapping_add(1)));
    }
    hash
}

pub struct X402Middleware {
    config: Arc<X402Config>,
    exclude_paths: Vec<String>,
}

impl X402Middleware {
    pub fn new(config: X402Config) -> Self {
        Self {
            config: Arc::new(config),
            exclude_paths: vec![
                "/docs".to_string(),
                "/swagger".to_string(),
                "/static".to_string(),
            ],
        }
    }

    pub fn exclude_paths(mut self, paths: Vec<String>) -> Self {
        self.exclude_paths.extend(paths);
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for X402Middleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = X402MiddlewareService<S>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>>>>;

    fn new_transform(&self, service: S) -> Self::Future {
        Box::pin(ok(X402MiddlewareService {
            service,
            config: self.config.clone(),
            exclude_paths: self.exclude_paths.clone(),
        }))
    }
}

pub struct X402MiddlewareService<S> {
    service: S,
    config: Arc<X402Config>,
    exclude_paths: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for X402MiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        
        let should_exclude = self.exclude_paths.iter().any(|p| path.starts_with(p));
        
        if should_exclude {
            let fut = self.service.call(req);
            return Box::pin(async move {
                Ok(fut.await?)
            });
        }

        let token = req.headers()
            .get("X-Payment-Token")
            .and_then(|v| v.to_str().ok());

        if token.is_some() && is_valid_token(token.unwrap()) {
            let fut = self.service.call(req);
            return Box::pin(async move {
                Ok(fut.await?)
            });
        }

        let config = self.config.clone();
        let payment_required = config.create_payment_required(&path);
        
        Box::pin(async move {
            let response = HttpResponse::PaymentRequired()
                .json(payment_required);
            
            let (req, _) = req.into_parts();
            Ok(ServiceResponse::new(req, response))
        })
    }
}

fn is_valid_token(token: &str) -> bool {
    if let Ok(decoded) = base64_decode(token) {
        let s = String::from_utf8_lossy(&decoded);
        return s.contains("\"signature\"") && s.contains("\"sender\"");
    }
    false
}

fn base64_decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(input)
}

pub fn require_payment(config: X402Config) -> X402Middleware {
    X402Middleware::new(config)
}
