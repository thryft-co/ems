use axum::{middleware as axum_middleware, routing::get, Router};
use std::{env, path::Path};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber;

use ems_server::{
    middleware::{auth::auth_middleware, tenant::tenant_middleware},
    routes::{assets, auth, items, jobs, machines, orders, persons, tenants},
    AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_environment();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize App State
    let app_state = AppState::new().await?;

    // Get static files directory from environment
    let static_files_dir = env::var("STATIC_FILES_DIR").unwrap_or_else(|_| "./static".to_string());

    // Build the application with routes and middleware
    let mut app = Router::new()
        .route("/health", get(health_check))
        // API routes
        .nest("/api/v1/auth", auth::routes())
        // Protected API routes (require auth and tenant isolation)
        .nest(
            "/api/v1/tenants",
            tenants::routes().layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            "/api/v1/person",
            persons::routes().layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            "/api/v1/job",
            jobs::routes().layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            "/api/v1/order",
            orders::routes().layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            "/api/v1/item",
            items::routes().layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            "/api/v1/asset",
            assets::routes().layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .nest(
            "/api/v1/machine",
            machines::routes().layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        );

    // Only add static file serving if the directory exists
    if Path::new(&static_files_dir).exists() {
        tracing::info!("Static files directory found: {}", static_files_dir);
        tracing::info!("Adding static file serving for frontend");
        app = app.fallback_service(
            ServeDir::new(&static_files_dir).append_index_html_on_directories(true),
        );
    } else {
        tracing::warn!("Static files directory not found: {}", static_files_dir);
        tracing::info!("Running in API-only mode (no frontend static files)");
        // Add a simple fallback for non-API routes when no static files exist
        app = app.fallback(api_only_fallback);
    }

    let app = app
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        // Tenant middleware should run before auth middleware
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            tenant_middleware,
        ))
        .with_state(app_state);

    // Start server
    let port = env::var("BACKEND_PORT")
        .unwrap_or_else(|_| env::var("PORT").unwrap_or_else(|_| "5002".to_string()));
    let address = format!("0.0.0.0:{}", port);

    tracing::info!("Server starting on {}", address);
    tracing::info!("API endpoints available at /api/*");

    if Path::new(&static_files_dir).exists() {
        tracing::info!("Frontend routes available at /*");
    } else {
        tracing::info!("Running in API-only mode - frontend not available");
    }

    let listener = tokio::net::TcpListener::bind(&address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn load_environment() {
    if let Ok(path) = env::var("ENV_FILE") {
        if Path::new(&path).exists() {
            dotenv::from_path(path).ok();
        }
    }

    for path in ["config/.env", "../../config/.env", ".env"] {
        if Path::new(path).exists() {
            dotenv::from_path(path).ok();
        }
    }
}

async fn health_check() -> &'static str {
    "OK"
}

// Fallback handler for when running without frontend static files
async fn api_only_fallback() -> (axum::http::StatusCode, &'static str) {
    (
        axum::http::StatusCode::NOT_FOUND,
        "API server running in standalone mode. Frontend not available. Use /api/* endpoints.",
    )
}
