use axum::{
    routing::{get, post},
    Router,
    Json,
};
use x402_agent_sdk::{
    adapters::axum::{x402_middleware, AxumX402Config},
    PaymentRequired,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    data: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct AnalysisRequest {
    input: String,
}

#[derive(Serialize, Deserialize)]
struct AnalysisResponse {
    result: String,
    input: String,
}

#[tokio::main]
async fn main() {
    let config = AxumX402Config::new(
        100,                                      // price_per_request
        "v1-eip155-exact",                        // scheme
        "eip155:1",                              // network  
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",  // payment_token (USDC)
        "0x742d35Cc6634C0532925a3b844Bc9e7595f0fB1E",  // recipient
    ).with_description("API Access");

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/data", get(data_handler))
        .route("/api/analyze", post(analyze_handler))
        .layer(axum::middleware::from_fn(move |req, next| {
            x402_middleware(config.clone(), req, next)
        }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("x402 Axum server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Welcome to x402 Axum Example",
        "x402_enabled": true
    }))
}

async fn data_handler() -> Json<ApiResponse> {
    Json(ApiResponse {
        data: "This is paid premium data!".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

async fn analyze_handler(Json(payload): Json<AnalysisRequest>) -> Json<AnalysisResponse> {
    Json(AnalysisResponse {
        result: "Analysis complete!".to_string(),
        input: payload.input,
    })
}
