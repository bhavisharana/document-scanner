use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod models;
mod routes;
mod services;

use config::Config;
use routes::{health_check, root, scan_document, AppState};
use services::OcrEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Tracing Subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "document_scanner=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Load Configuration
    let config = Config::from_env();
    tracing::info!("Starting Rust Document Scanner service on port {}", config.port);
    tracing::info!("Using Tesseract binary: {}", config.tesseract_path);

    // 3. Initialize Shared Application State
    let ocr = Arc::new(OcrEngine::new(
        config.tesseract_path.clone(),
        config.tessdata_prefix.clone(),
    ));

    let state = Arc::new(AppState {
        config: config.clone(),
        ocr,
    });

    // 4. Configure Permissive CORS for seamless Web Frontend access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let max_body_limit = config.max_file_size_bytes;

    // 5. Build Axum Router (Serving Vue 3 frontend if dist exists)
    let app = if std::path::Path::new("frontend/dist").exists() {
        tracing::info!("Serving built Vue 3 frontend from frontend/dist");
        let serve_dir = ServeDir::new("frontend/dist")
            .not_found_service(ServeFile::new("frontend/dist/index.html"));

        Router::new()
            .route("/api/health", get(health_check))
            .route("/api/documents/scan", post(scan_document))
            .fallback_service(serve_dir)
            .layer(DefaultBodyLimit::max(max_body_limit))
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    } else {
        Router::new()
            .route("/", get(root))
            .route("/api/health", get(health_check))
            .route("/api/documents/scan", post(scan_document))
            .layer(DefaultBodyLimit::max(max_body_limit))
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    };

    // 6. Bind Listener & Serve
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Document Scanner server listening on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
