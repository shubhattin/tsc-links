//! App initialisation + router composition.
//!
//! - `/api/*`: admin JSON API ([`crate::routes::links`]), every route behind
//!   [`crate::auth::require_admin`] (Better Auth session cookie, admin role).
//!   Registered on a [`utoipa_axum::OpenApiRouter`]; `openapi_router` (no
//!   state) feeds the OpenAPI document.
//! - `/{slug}`: public redirects ([`crate::routes::redirect`]).
//! - Anything else: JSON 404 under `/api`, HTML 404 elsewhere.
//!
//! On Vercel, static files (the SvelteKit page at `/`, `/_app/*`,
//! `/favicon.ico`, …) are served first; only the remaining paths reach here.

use crate::state::AppState;
use crate::utils::cache::LinkCache;
use crate::utils::config::EnvConfig;
use axum::Router;
use axum::http::{StatusCode, Uri};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use std::time::Duration;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

/// Load env, then connect Postgres and Redis concurrently (cold starts matter
/// on Vercel). Redis failures only disable the cache.
pub async fn init_state() -> Result<AppState, String> {
  let _ = dotenvy::dotenv();
  let _ = dotenvy::from_filename(".env.local");

  let config = EnvConfig::from_env()?;
  let database_url = std::env::var("PG_DATABASE_URL")
    .map_err(|_| "PG_DATABASE_URL must be set: missing environment variable".to_string())?;

  let (db, cache) = tokio::join!(
    crate::utils::db::connect(&database_url),
    LinkCache::connect(config.redis_url.as_deref()),
  );
  let db = db.map_err(|e| format!("failed to connect to database: {e}"))?;

  let http = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .build()
    .map_err(|e| format!("failed to build HTTP client: {e}"))?;

  Ok(AppState {
    db,
    cache,
    http,
    config: Arc::new(config),
  })
}

async fn fallback(uri: Uri) -> Response {
  let path = uri.path();
  if path == "/api" || path.starts_with("/api/") {
    crate::auth::error_response(StatusCode::NOT_FOUND, format!("Not found: {path}"))
  } else {
    crate::routes::redirect::not_found_page().into_response()
  }
}

fn compose_openapi_router(state: Option<&AppState>) -> OpenApiRouter<AppState> {
  let protected = OpenApiRouter::new()
    .merge(crate::routes::links::openapi_router())
    .merge(crate::routes::expenses::openapi_router());
  let protected = match state {
    Some(state) => protected.route_layer(middleware::from_fn_with_state(
      state.clone(),
      crate::auth::require_admin,
    )),
    None => protected,
  };
  OpenApiRouter::with_openapi(crate::openapi::ApiDoc::openapi()).merge(protected)
}

/// Composed OpenAPI-aware router (used for the OpenAPI document only).
pub fn openapi_router() -> OpenApiRouter<AppState> {
  compose_openapi_router(None)
}

/// Axum router (no Vercel-specific layers).
pub fn router(state: AppState) -> Router {
  let api: Router<AppState> = compose_openapi_router(Some(&state)).into();
  api
    .merge(crate::routes::redirect::router())
    .fallback(fallback)
    .with_state(state)
}
