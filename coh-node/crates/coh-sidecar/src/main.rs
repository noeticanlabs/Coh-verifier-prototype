mod error;
mod llm_proxy;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // Initializing tracing (simple stdout logging)
    tracing_subscriber::fmt::init();

    let host = std::env::var("COH_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("COH_PORT").unwrap_or_else(|_| "3030".to_string());
    let upstream_url = std::env::var("COH_SIDECAR_UPSTREAM_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());

    // Create shared app state for LLM proxy
    let app_state = std::sync::Arc::new(llm_proxy::AppState::new(upstream_url));

    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str.parse().expect("Invalid address");

    let app = Router::new()
        // Health check
        .route("/health", get(routes::health_check))
        // Legacy Coh endpoints
        .route("/verify/micro", post(routes::verify_micro_handler))
        .route("/verify/chain", post(routes::verify_chain_handler))
        .route(
            "/execute/verified",
            post(routes::execute_verified_handler),
        )
        .route(
            "/trajectory/search",
            post(routes::trajectory_search_handler),
        )
        // OpenAI-compatible LLM proxy endpoints
        .route(
            "/v1/chat/completions",
            post(llm_proxy::chat_completions_handler),
        )
        .route("/v1/models", get(llm_proxy::models_handler))
        // Coh-specific endpoints
        .route("/coh/stats", get(llm_proxy::coh_stats_handler))
        .route("/coh/reset", post(llm_proxy::coh_reset_handler))
        .route("/coh/chain", get(llm_proxy::coh_chain_handler))
        .with_state(app_state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    println!("🚀 Coh Sidecar active at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
