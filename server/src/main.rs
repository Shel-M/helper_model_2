mod config;

use std::{fs::File, sync::Arc};

use axum::{routing::get, Router};
use chrono::Datelike;
use tokio::{net::TcpListener, signal, sync::RwLock};
use tracing::{debug, info};

type DB = sqlx::SqlitePool;
pub struct AppData {
    pub db: DB,
}

#[tokio::main]
async fn main() {
    let config = config::Config::new().unwrap();
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .init();

    info!("Starting server...");

    let time = chrono::Utc::now().num_days_from_ce();
    info!("{time}");

    let db = connect_to_db(&config).await.unwrap();

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let router = router(AppData { db });
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown())
        .await
        .expect("Could not start server");
}

fn router(app_data: AppData) -> Router {
    let shared_data = Arc::new(RwLock::new(app_data));

    debug!("Creating router");

    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .with_state(shared_data)
}

async fn connect_to_db(config: &config::Config) -> Result<DB, sqlx::Error> {
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.get_db_url())
        .await;

    match db {
        Ok(db) => {
            info!("Connected to database");
            Ok(db)
        }
        Err(e) if e.to_string().contains("code: 14") => {
            info!("Failed to connect to database, does not exist. Creating...");

            let mut filename = config.database.clone();
            if !filename.ends_with(".db") {
                filename = filename + ".db";
            }

            File::create(filename).expect("Failed to create database file");

            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect(&config.get_db_url())
                .await
        }
        Err(e) => Err(e),
    }
}

async fn shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    };
}
