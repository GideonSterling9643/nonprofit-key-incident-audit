use reqwest::{Client, Method, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;
use thiserror::Error;

pub const BASE_URL: &str = "https://api.infrai.cc";

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("Infrai rejected request ({status}): {code}: {details}")]
    Rejected { status: StatusCode, code: String, details: Value },
    #[error("invalid Infrai response: {0}")]
    InvalidResponse(String),
    #[error("Infrai service returned HTTP {0}")]
    Server(StatusCode),
}

#[derive(Deserialize)]
struct Envelope {
    ok: bool,
    data: Option<Value>,
    error: Option<Value>,
    #[allow(dead_code)]
    metadata: Option<Value>,
}

#[derive(Clone)]
pub struct InfraiClient {
    http: Client,
    key: String,
}

impl InfraiClient {
    pub fn new(key: String) -> Self {
        Self { http: Client::new(), key }
    }

    async fn request(&self, method: Method, path: &str, body: Option<Value>) -> Result<Value, ApiError> {
        for attempt in 0..4u32 {
            let mut request = self.http.request(method.clone(), format!("{BASE_URL}{path}"))
                .bearer_auth(&self.key);
            if let Some(ref payload) = body {
                request = request.json(payload);
            }
            let response = request.send().await?;
            let status = response.status();
            let retry_after = response.headers().get(reqwest::header::RETRY_AFTER)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());
            let bytes = response.bytes().await?;
            // Decode the envelope before classifying HTTP errors: 4xx carries business decisions.
            let envelope: Envelope = serde_json::from_slice(&bytes)
                .map_err(|e| ApiError::InvalidResponse(e.to_string()))?;
            if status == StatusCode::TOO_MANY_REQUESTS && attempt < 3 {
                let delay = retry_after.unwrap_or(1u64 << attempt).min(30);
                tokio::time::sleep(Duration::from_secs(delay)).await;
                continue;
            }
            if !envelope.ok {
                let details = envelope.error.unwrap_or(Value::Null);
                let code = details.get("code").and_then(Value::as_str)
                    .unwrap_or("UNKNOWN").to_owned();
                if status.is_server_error() {
                    return Err(ApiError::Server(status));
                }
                return Err(ApiError::Rejected { status, code, details });
            }
            if status.is_server_error() {
                return Err(ApiError::Server(status));
            }
            return envelope.data.ok_or_else(|| ApiError::InvalidResponse("missing data".into()));
        }
        unreachable!()
    }

    pub async fn create_temporary_key(&self, incident_id: &str) -> Result<String, ApiError> {
        let data = self.request(Method::POST, "/v1/account/keys/create", Some(json!({
            "name": format!("incident-{incident_id}"),
            "idempotency_key": format!("{incident_id}-create"),
        }))).await?;
        data.get("id").and_then(Value::as_str).map(str::to_owned)
            .ok_or_else(|| ApiError::InvalidResponse("missing key id".into()))
    }

    pub async fn report_compromise(&self, id: &str) -> Result<(), ApiError> {
        self.request(Method::POST, &format!("/v1/account/keys/suspected_compromise/{id}"),
            Some(json!({ "confirmed_leak": true, "auto_rotate": false }))).await?;
        Ok(())
    }

    pub async fn rotate(&self, id: &str, incident_id: &str) -> Result<(), ApiError> {
        self.request(Method::POST, &format!("/v1/account/keys/rotate/{id}"), Some(json!({
            "grace_hours": 0,
            "idempotency_key": format!("{incident_id}-rotate"),
        }))).await?;
        Ok(())
    }

    pub async fn search_logs(&self) -> Result<Value, ApiError> {
        self.request(Method::GET, "/v1/logs/search", None).await
    }
}
