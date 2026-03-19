use wasm_bindgen::prelude::*;

use crate::chains::{
    SolanaNetwork, SolanaPayment, SolanaToken, StacksNetwork, StacksPayment, StacksToken,
};
use crate::core::credential::{Credential, CredentialBuilder};
use crate::core::payment::{PaymentRequired, PaymentToken};

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct WasmPaymentRequired {
    inner: PaymentRequired,
}

#[wasm_bindgen]
impl WasmPaymentRequired {
    #[wasm_bindgen(constructor)]
    pub fn new(
        scheme: &str,
        network: &str,
        payment_token: &str,
        max_amount: &str,
        recipient: &str,
    ) -> WasmPaymentRequired {
        WasmPaymentRequired {
            inner: PaymentRequired::new(scheme, network, payment_token, max_amount, recipient),
        }
    }

    #[wasm_bindgen]
    pub fn with_expiry(mut self, expires_at: &str) -> Result<WasmPaymentRequired, JsValue> {
        let dt = chrono::DateTime::parse_from_rfc3339(expires_at)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner = self.inner.with_expiry(dt.with_timezone(&chrono::Utc));
        Ok(self)
    }

    #[wasm_bindgen]
    pub fn with_description(mut self, description: &str) -> WasmPaymentRequired {
        self.inner = self.inner.with_description(description);
        self
    }

    #[wasm_bindgen]
    pub fn to_header_value(&self) -> Result<String, JsValue> {
        self.inner
            .to_header_value()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn from_header_value(value: &str) -> Result<WasmPaymentRequired, JsValue> {
        PaymentRequired::from_header_value(value)
            .map(|inner| WasmPaymentRequired { inner })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(getter)]
    pub fn version(&self) -> u32 {
        self.inner.version
    }

    #[wasm_bindgen(getter)]
    pub fn scheme(&self) -> String {
        self.inner.scheme.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn network(&self) -> String {
        self.inner.network.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn payment_token(&self) -> String {
        self.inner.payment_token.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn max_amount(&self) -> String {
        self.inner.max_amount.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn recipient(&self) -> String {
        self.inner.recipient.clone()
    }
}

#[wasm_bindgen]
pub struct WasmPaymentToken {
    inner: PaymentToken,
}

#[wasm_bindgen]
impl WasmPaymentToken {
    #[wasm_bindgen]
    pub fn encode(&self) -> Result<String, JsValue> {
        self.inner
            .encode()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn decode(encoded: &str) -> Result<WasmPaymentToken, JsValue> {
        PaymentToken::decode(encoded)
            .map(|inner| WasmPaymentToken { inner })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn is_expired(&self) -> bool {
        self.inner.is_expired()
    }
}

#[wasm_bindgen]
pub struct WasmCredential {
    inner: Credential,
}

#[wasm_bindgen]
impl WasmCredential {
    #[wasm_bindgen]
    pub fn new(issuer: &str, subject: &str, claims_json: &str) -> Result<WasmCredential, JsValue> {
        let claims: serde_json::Value =
            serde_json::from_str(claims_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(WasmCredential {
            inner: Credential::new(issuer, subject, claims),
        })
    }

    #[wasm_bindgen]
    pub fn with_expiry(mut self, expires_at: &str) -> Result<WasmCredential, JsValue> {
        let dt = chrono::DateTime::parse_from_rfc3339(expires_at)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner = self.inner.with_expiry(dt.with_timezone(&chrono::Utc));
        Ok(self)
    }

    #[wasm_bindgen]
    pub fn with_type(mut self, credential_type: &str) -> WasmCredential {
        self.inner = self.inner.with_type(credential_type);
        self
    }

    #[wasm_bindgen]
    pub fn is_expired(&self) -> bool {
        self.inner.is_expired()
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[wasm_bindgen]
pub struct WasmCredentialBuilder {
    inner: CredentialBuilder,
}

#[wasm_bindgen]
impl WasmCredentialBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(issuer: &str, subject: &str) -> WasmCredentialBuilder {
        WasmCredentialBuilder {
            inner: CredentialBuilder::new(issuer, subject),
        }
    }

    #[wasm_bindgen]
    pub fn claim(mut self, key: &str, value: JsValue) -> Result<WasmCredentialBuilder, JsValue> {
        let val: serde_json::Value =
            serde_wasm_bindgen::from_value(value).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner = self.inner.claim(key, val);
        Ok(self)
    }

    #[wasm_bindgen]
    pub fn expires_at(mut self, expires_at: &str) -> Result<WasmCredentialBuilder, JsValue> {
        let dt = chrono::DateTime::parse_from_rfc3339(expires_at)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner = self.inner.expires_at(dt.with_timezone(&chrono::Utc));
        Ok(self)
    }

    #[wasm_bindgen]
    pub fn build(self) -> WasmCredential {
        WasmCredential {
            inner: self.inner.build(),
        }
    }
}

#[wasm_bindgen]
pub struct WasmSolanaPayment {
    inner: SolanaPayment,
}

#[wasm_bindgen]
impl WasmSolanaPayment {
    #[wasm_bindgen(constructor)]
    pub fn new(
        network: &str,
        recipient: &str,
        token: &str,
        amount: u64,
    ) -> Result<WasmSolanaPayment, JsValue> {
        let net = match network {
            "mainnet" => SolanaNetwork::Mainnet,
            "devnet" => SolanaNetwork::Devnet,
            "testnet" => SolanaNetwork::Testnet,
            _ => return Err(JsValue::from_str("Invalid network")),
        };

        let tok = match token {
            "SOL" => SolanaToken::SOL,
            "USDC" => SolanaToken::USDC,
            "USDT" => SolanaToken::USDT,
            _ => return Err(JsValue::from_str("Invalid token")),
        };

        Ok(WasmSolanaPayment {
            inner: SolanaPayment::new(net, recipient, tok, amount),
        })
    }

    #[wasm_bindgen]
    pub fn to_payment_required(&self) -> WasmPaymentRequired {
        WasmPaymentRequired {
            inner: self.inner.to_payment_required(),
        }
    }
}

#[wasm_bindgen]
pub struct WasmStacksPayment {
    inner: StacksPayment,
}

#[wasm_bindgen]
impl WasmStacksPayment {
    #[wasm_bindgen(constructor)]
    pub fn new(
        network: &str,
        recipient: &str,
        token: &str,
        amount: u64,
    ) -> Result<WasmStacksPayment, JsValue> {
        let net = match network {
            "mainnet" => StacksNetwork::Mainnet,
            "testnet" => StacksNetwork::Testnet,
            "devnet" => StacksNetwork::Devnet,
            _ => return Err(JsValue::from_str("Invalid network")),
        };

        let tok = match token {
            "STX" => StacksToken::STX,
            "SIP010" => StacksToken::SIP010,
            _ => return Err(JsValue::from_str("Invalid token")),
        };

        Ok(WasmStacksPayment {
            inner: StacksPayment::new(net, recipient, tok, amount),
        })
    }

    #[wasm_bindgen]
    pub fn to_payment_required(&self) -> WasmPaymentRequired {
        WasmPaymentRequired {
            inner: self.inner.to_payment_required(),
        }
    }
}
