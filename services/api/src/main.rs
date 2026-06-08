mod config;
mod db;
mod error;
mod repositories;
mod routes;
mod state;

use config::Config;
use state::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "csqd_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let db = db::connect(&config)
        .await
        .expect("database connection should be available");
    let state = AppState::new(db);

    let listener = tokio::net::TcpListener::bind(config.api_addr)
        .await
        .expect("API listener should bind");

    tracing::info!("C-SQD API listening on http://{}", config.api_addr);

    axum::serve(listener, routes::router(state))
        .await
        .expect("API should serve");
}
