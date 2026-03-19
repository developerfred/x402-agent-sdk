use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub r#type: Vec<String>,
    pub issuer: String,
    pub subject: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub claims: serde_json::Value,
}

impl Credential {
    pub fn new(issuer: &str, subject: &str, claims: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            r#type: vec!["VerifiableCredential".to_string()],
            issuer: issuer.to_string(),
            subject: subject.to_string(),
            issued_at: Utc::now(),
            expires_at: None,
            claims,
        }
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn with_type(mut self, credential_type: &str) -> Self {
        self.r#type.push(credential_type.to_string());
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = &self.expires_at {
            Utc::now() > *expires_at
        } else {
            false
        }
    }
}

pub struct CredentialBuilder {
    issuer: String,
    subject: String,
    claims: serde_json::Value,
    credential_type: Option<String>,
    expires_at: Option<DateTime<Utc>>,
}

impl Clone for CredentialBuilder {
    fn clone(&self) -> Self {
        Self {
            issuer: self.issuer.clone(),
            subject: self.subject.clone(),
            claims: self.claims.clone(),
            credential_type: self.credential_type.clone(),
            expires_at: self.expires_at,
        }
    }
}

impl CredentialBuilder {
    pub fn new(issuer: &str, subject: &str) -> Self {
        Self {
            issuer: issuer.to_string(),
            subject: subject.to_string(),
            claims: serde_json::json!({}),
            credential_type: None,
            expires_at: None,
        }
    }

    pub fn claim(mut self, key: &str, value: serde_json::Value) -> Self {
        self.claims[key] = value;
        self
    }

    pub fn credential_type(mut self, r#type: &str) -> Self {
        self.credential_type = Some(r#type.to_string());
        self
    }

    pub fn expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn build(self) -> Credential {
        let mut cred = Credential::new(&self.issuer, &self.subject, self.claims);

        if let Some(exp) = self.expires_at {
            cred = cred.with_expiry(exp);
        }

        if let Some(ct) = self.credential_type {
            cred = cred.with_type(&ct);
        }

        cred
    }
}
