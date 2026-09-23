mod incident;
mod infrai_client;

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use incident::Incident;
use infrai_client::InfraiClient;
use serde_json::{json, Value};

async fn audit(
    State(_client): State<InfraiClient>,
    Json(incident): Json<Incident>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if incident.incident_id.is_empty()
        || !incident
            .incident_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "incident_id must be alphanumeric or hyphen" })),
        ));
    }
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "account.keys.create is disabled because the available contract has no key deletion capability"
        })),
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::var("INFRAI_API_KEY")?;
    let app = Router::new()
        .route("/incident", post(audit))
        .with_state(InfraiClient::new(key));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn incident_is_rejected_before_creating_an_undeletable_key() {
        let result = audit(
            State(InfraiClient::new("unused".into())),
            Json(Incident {
                incident_id: "incident-test".into(),
                affected_workflows: Vec::new(),
            }),
        )
        .await;

        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    }
}
