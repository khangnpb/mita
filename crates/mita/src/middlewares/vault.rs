use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_auth::AuthBearer;
use reqwest::StatusCode;
use thiserror::Error;

use crate::{
    app_state::AppState,
    vault::{self, VaultError},
};

pub async fn authenticate<B>(
    state: State<AppState>,
    id_token: AuthBearer,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AuthError> {
    let vault =
        vault::Client::login(&state.0.http_client, &state.0.config.vault.url, &id_token.0).await?;
    req.extensions_mut().insert(vault);
    Ok(next.run(req).await)
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct AuthError(#[from] VaultError);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self.0 {
            VaultError::Unexpected(e) => {
                tracing::error!("unexpected error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json("unexpected error")).into_response()
            }
            VaultError::Status(status, errors) => {
                tracing::error!("status {}, errors: {:?}", status, errors);
                (status, Json(serde_json::json!({ "errors": errors }))).into_response()
            }
        }
    }
}