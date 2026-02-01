use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{patch, post},
    Json, Router,
};
use serde::Deserialize;
use tracing::{debug, error};

use crate::{get_db, user::User, SharedState};

pub fn user_router() -> Router<SharedState> {
    Router::new()
        .route("/", post(new_user))
        .route("/", patch(update_user))
}

#[derive(Deserialize)]
pub struct NewUser {
    pub name: String,
    pub discord_tag: Option<String>,
}

pub async fn new_user(
    State(shared_state): State<SharedState>,
    Json(payload): Json<NewUser>,
) -> impl IntoResponse {
    let db = get_db(shared_state).await;

    // Add check for if the user name already exists, if it does, send error
    let user = User::new(&payload.name, payload.discord_tag);
    match user.insert(&db).await {
        Ok(u) => {
            debug!("New user inserted {u:?}");
            (StatusCode::OK, "".into())
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e).to_string()),
    }
}

#[derive(Deserialize)]
pub struct UpdateUser {
    pub id: i64,
    pub name: Option<String>,
    pub discord_tag: Option<String>,
}

pub async fn update_user(
    State(shared_state): State<SharedState>,
    Json(update): Json<UpdateUser>,
) -> impl IntoResponse {
    let db = get_db(shared_state).await;

    let Ok(user) = User::get_by_id(&db, update.id).await else {
        return StatusCode::NOT_FOUND;
    };

    if let Err(e) = user.update(&db, update).await {
        error!("Error updating user '{e}'");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
