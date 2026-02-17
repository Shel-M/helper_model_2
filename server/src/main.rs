mod api;
mod config;
mod user;

#[cfg(test)]
mod tests;

use std::{fs::File, sync::Arc};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::Datelike;
use sqlx::query_as;
use tokio::{net::TcpListener, signal, sync::RwLock};
use tracing::{debug, info};

use crate::{api::user::user_router, user::User};

type DB = sqlx::SqlitePool;

pub struct AppData {
    pub db: DB,
}
pub type SharedState = Arc<RwLock<AppData>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Config::new()?;
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .init();

    info!("Starting server...");

    let time = chrono::Utc::now().num_days_from_ce();
    info!("{time}");

    let db = connect_to_db(&config).await?;

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    setup_server(db).await
}

async fn setup_server(db: DB) -> anyhow::Result<()> {
    let router = router(AppData { db });
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    debug!("Starting server...");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown())
        .await
        .expect("Could not start server");
    Ok(())
}

fn router(app_data: AppData) -> Router {
    let shared_data = Arc::new(RwLock::new(app_data));

    debug!("Creating router");

    Router::new()
        .route("/users", get(users))
        .route("/delete/{id}", get(delete))
        .nest("/api/v0/user", user_router())
        .with_state(shared_data)
}

async fn users(State(shared_state): State<SharedState>) -> Result<Json<Vec<User>>, &'static str> {
    let db = get_db(shared_state).await;

    let Ok(users) = query_as!(User, "select * from person").fetch_all(&db).await else {
        return Err("Failed returning user list");
    };

    Ok(Json(users))
}

async fn delete(State(app_data): State<SharedState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = app_data.read().await.db.clone();
    let user = User::get_by_id(&db, id).await.expect("Couldn't get users");
    if let Err(e) = user.delete_ref(&db).await {
        return format!(
            "Couldn't delete user '{}' id: {} - {e:?}",
            user.name, user.id
        );
    }

    "Ok!".into()
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
            if !std::path::Path::new(&filename)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("db"))
            {
                filename += ".db";
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

pub async fn get_db(shared_state: SharedState) -> DB {
    shared_state.read().await.db.clone()
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

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    };
}
