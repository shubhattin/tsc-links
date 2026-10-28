//! Admin links API (all routes behind [`crate::auth::require_admin`]).
//!
//! | Method | Path | Handler |
//! |--------|------|---------|
//! | `GET` | `/api/links` | [`list_links`] |
//! | `POST` | `/api/links` | [`create_link`] |
//! | `PATCH` | `/api/links/{id}` | [`update_link`] |
//! | `DELETE` | `/api/links/{id}` | [`delete_link`] |
//!
//! Every write also updates the Redis redirect cache, so edits take effect on
//! the very next redirect.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::auth::{AuthUser, ErrorBody, error_response};
use crate::entities::links;
use crate::state::AppState;
use crate::utils::cache::CachedLink;
use crate::utils::id::new_id;
use crate::utils::slug::{generate_slug, validate_slug};

const URL_MAX_LEN: usize = 4096;
const TITLE_MAX_LEN: usize = 255;
/// Attempts at a fresh random slug before giving up (collisions are ~never).
const GENERATED_SLUG_ATTEMPTS: usize = 5;

/// Link row for API responses (timestamps as RFC 3339 strings).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LinkDto {
  pub id: String,
  /// Public path segment: the short link is `<origin>/<slug>`.
  pub slug: String,
  /// Redirect target.
  pub url: String,
  pub title: Option<String>,
  pub enabled: bool,
  pub created_by: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

impl From<links::Model> for LinkDto {
  fn from(m: links::Model) -> Self {
    Self {
      id: m.id,
      slug: m.slug,
      url: m.url,
      title: m.title,
      enabled: m.enabled,
      created_by: m.created_by,
      created_at: m.created_at.to_rfc3339(),
      updated_at: m.updated_at.to_rfc3339(),
    }
  }
}

/// `GET /api/links` response (newest first).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LinksListResponse {
  pub links: Vec<LinkDto>,
}

/// `POST /api/links` body.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLinkBody {
  /// Absolute http(s) URL.
  pub url: String,
  /// Custom slug; omitted/empty → random 7-char nanoid.
  pub slug: Option<String>,
  /// Optional label (max 255 chars).
  pub title: Option<String>,
  /// Default `true`.
  pub enabled: Option<bool>,
}

/// `PATCH /api/links/{id}` body — omitted fields are left unchanged.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLinkBody {
  pub url: Option<String>,
  pub slug: Option<String>,
  /// Empty string clears the title.
  pub title: Option<String>,
  pub enabled: Option<bool>,
}

fn bad_request(message: impl Into<String>) -> Response {
  error_response(StatusCode::BAD_REQUEST, message)
}

fn internal_error(e: DbErr) -> Response {
  eprintln!("database error: {e}");
  error_response(StatusCode::INTERNAL_SERVER_ERROR, "database error")
}

fn is_unique_violation(e: &DbErr) -> bool {
  matches!(
    e.sql_err(),
    Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
  )
}

fn slug_taken(slug: &str) -> Response {
  error_response(
    StatusCode::CONFLICT,
    format!("slug '{slug}' is already in use"),
  )
}

/// Trim + validate an absolute http(s) URL; returns its normalized form
/// (ASCII-only, so it is always a valid `Location` header value).
fn normalize_url(raw: &str) -> Result<String, String> {
  let raw = raw.trim();
  if raw.is_empty() {
    return Err("url is required".to_string());
  }
  if raw.len() > URL_MAX_LEN {
    return Err(format!("url must be at most {URL_MAX_LEN} characters"));
  }
  let parsed = url::Url::parse(raw).map_err(|_| "url must be a valid absolute URL".to_string())?;
  if !matches!(parsed.scheme(), "http" | "https") {
    return Err("url must start with http:// or https://".to_string());
  }
  if parsed.host_str().is_none_or(str::is_empty) {
    return Err("url must include a host".to_string());
  }
  Ok(parsed.to_string())
}

/// Trim; empty → `None`.
fn normalize_title(raw: Option<&str>) -> Result<Option<String>, String> {
  let Some(title) = raw.map(str::trim).filter(|t| !t.is_empty()) else {
    return Ok(None);
  };
  if title.chars().count() > TITLE_MAX_LEN {
    return Err(format!("title must be at most {TITLE_MAX_LEN} characters"));
  }
  Ok(Some(title.to_string()))
}

/// Redirect-cache state for a row.
fn cache_state(row: &links::Model) -> CachedLink {
  if row.enabled {
    CachedLink::Url(row.url.clone())
  } else {
    CachedLink::Missing
  }
}

/// Link routes with OpenAPI path registration.
pub fn openapi_router() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes!(list_links, create_link))
    .routes(routes!(update_link, delete_link))
}

/// `GET /api/links` — every link, newest first.
#[utoipa::path(
    get,
    path = "/api/links",
    operation_id = "links.list",
    tag = "links",
    security(("session_cookie" = [])),
    responses(
        (status = 200, description = "All links", body = LinksListResponse),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
    )
)]
pub async fn list_links(State(state): State<AppState>) -> Response {
  match links::Entity::find()
    .order_by_desc(links::Column::CreatedAt)
    .all(&state.db)
    .await
  {
    Ok(rows) => Json(LinksListResponse {
      links: rows.into_iter().map(LinkDto::from).collect(),
    })
    .into_response(),
    Err(e) => internal_error(e),
  }
}

/// `POST /api/links` — create a link (random slug unless one is given).
#[utoipa::path(
    post,
    path = "/api/links",
    operation_id = "links.create",
    tag = "links",
    security(("session_cookie" = [])),
    request_body = CreateLinkBody,
    responses(
        (status = 201, description = "Link created", body = LinkDto),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
        (status = 409, description = "Slug already in use", body = ErrorBody),
    )
)]
pub async fn create_link(
  State(state): State<AppState>,
  auth: AuthUser,
  Json(body): Json<CreateLinkBody>,
) -> Response {
  let url = match normalize_url(&body.url) {
    Ok(u) => u,
    Err(msg) => return bad_request(msg),
  };
  let title = match normalize_title(body.title.as_deref()) {
    Ok(t) => t,
    Err(msg) => return bad_request(msg),
  };
  let custom_slug = body
    .slug
    .as_deref()
    .map(str::trim)
    .filter(|s| !s.is_empty());
  if let Some(slug) = custom_slug
    && let Err(msg) = validate_slug(slug)
  {
    return bad_request(msg);
  }

  let attempts = if custom_slug.is_some() {
    1
  } else {
    GENERATED_SLUG_ATTEMPTS
  };
  for _ in 0..attempts {
    let slug = custom_slug.map_or_else(generate_slug, str::to_string);
    let now = chrono::Utc::now().fixed_offset();
    let model = links::ActiveModel {
      id: Set(new_id()),
      slug: Set(slug.clone()),
      url: Set(url.clone()),
      title: Set(title.clone()),
      enabled: Set(body.enabled.unwrap_or(true)),
      created_by: Set(Some(auth.user_id.clone())),
      created_at: Set(now),
      updated_at: Set(now),
    };
    match model.insert(&state.db).await {
      Ok(row) => {
        // Also overwrites a negative entry left by earlier visits to this slug.
        state.cache.put(&row.slug, &cache_state(&row)).await;
        return (StatusCode::CREATED, Json(LinkDto::from(row))).into_response();
      }
      Err(e) if is_unique_violation(&e) => {
        if custom_slug.is_some() {
          return slug_taken(&slug);
        }
      }
      Err(e) => return internal_error(e),
    }
  }
  error_response(
    StatusCode::INTERNAL_SERVER_ERROR,
    "could not generate a unique slug, try again",
  )
}

/// `PATCH /api/links/{id}` — edit url / slug / title / enabled.
#[utoipa::path(
    patch,
    path = "/api/links/{id}",
    operation_id = "links.update",
    tag = "links",
    security(("session_cookie" = [])),
    params(("id" = String, Path, description = "Link id (nanoid)")),
    request_body = UpdateLinkBody,
    responses(
        (status = 200, description = "Link updated", body = LinkDto),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
        (status = 404, description = "Link not found", body = ErrorBody),
        (status = 409, description = "Slug already in use", body = ErrorBody),
    )
)]
pub async fn update_link(
  State(state): State<AppState>,
  Path(id): Path<String>,
  Json(body): Json<UpdateLinkBody>,
) -> Response {
  let row = match links::Entity::find_by_id(&id).one(&state.db).await {
    Ok(Some(row)) => row,
    Ok(None) => return error_response(StatusCode::NOT_FOUND, "link not found"),
    Err(e) => return internal_error(e),
  };
  let old_slug = row.slug.clone();
  let mut model: links::ActiveModel = row.into();

  if let Some(raw) = body.url.as_deref() {
    match normalize_url(raw) {
      Ok(url) => model.url = Set(url),
      Err(msg) => return bad_request(msg),
    }
  }
  if let Some(raw) = body.slug.as_deref() {
    let slug = raw.trim();
    if let Err(msg) = validate_slug(slug) {
      return bad_request(msg);
    }
    model.slug = Set(slug.to_string());
  }
  if body.title.is_some() {
    match normalize_title(body.title.as_deref()) {
      Ok(title) => model.title = Set(title),
      Err(msg) => return bad_request(msg),
    }
  }
  if let Some(enabled) = body.enabled {
    model.enabled = Set(enabled);
  }
  model.updated_at = Set(chrono::Utc::now().fixed_offset());

  let row = match model.update(&state.db).await {
    Ok(row) => row,
    Err(e) if is_unique_violation(&e) => {
      return slug_taken(body.slug.as_deref().unwrap_or_default().trim());
    }
    Err(e) => return internal_error(e),
  };

  if row.slug != old_slug {
    state.cache.invalidate(&old_slug).await;
  }
  state.cache.put(&row.slug, &cache_state(&row)).await;
  Json(LinkDto::from(row)).into_response()
}

/// `DELETE /api/links/{id}` — delete a link (its slug becomes free again).
#[utoipa::path(
    delete,
    path = "/api/links/{id}",
    operation_id = "links.delete",
    tag = "links",
    security(("session_cookie" = [])),
    params(("id" = String, Path, description = "Link id (nanoid)")),
    responses(
        (status = 204, description = "Link deleted"),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
        (status = 404, description = "Link not found", body = ErrorBody),
    )
)]
pub async fn delete_link(State(state): State<AppState>, Path(id): Path<String>) -> Response {
  let row = match links::Entity::find_by_id(&id).one(&state.db).await {
    Ok(Some(row)) => row,
    Ok(None) => return error_response(StatusCode::NOT_FOUND, "link not found"),
    Err(e) => return internal_error(e),
  };
  if let Err(e) = links::Entity::delete_by_id(&row.id).exec(&state.db).await {
    return internal_error(e);
  }
  state.cache.invalidate(&row.slug).await;
  StatusCode::NO_CONTENT.into_response()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn url_normalization() {
    assert_eq!(
      normalize_url("  https://example.com  ").unwrap(),
      "https://example.com/"
    );
    assert_eq!(
      normalize_url("https://例え.jp/パス?q=1").unwrap(),
      "https://xn--r8jz45g.jp/%E3%83%91%E3%82%B9?q=1"
    );
    for bad in ["", "example.com", "javascript:alert(1)", "ftp://x.com", "https://"] {
      assert!(normalize_url(bad).is_err(), "{bad} should be rejected");
    }
  }

  #[test]
  fn title_normalization() {
    assert_eq!(normalize_title(Some("  ")).unwrap(), None);
    assert_eq!(normalize_title(Some(" Hi ")).unwrap().as_deref(), Some("Hi"));
    assert!(normalize_title(Some(&"x".repeat(256))).is_err());
  }
}
