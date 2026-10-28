//! Public short-link redirects: `GET /{slug}` → `302 Found` to the target.
//!
//! Lookup order: Redis ([`crate::utils::cache`]) → Postgres (then cached).
//! Missing, disabled or malformed slugs get a small HTML 404 page. Kept out
//! of the OpenAPI document — this is browser traffic, not an API.

use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::entities::links;
use crate::state::AppState;
use crate::utils::cache::CachedLink;
use crate::utils::slug::validate_slug;

/// `/{slug}` and `/{slug}/` (HEAD is served by the same handler).
pub fn router() -> Router<AppState> {
  Router::new()
    .route("/{slug}", get(redirect))
    .route("/{slug}/", get(redirect))
}

async fn redirect(State(state): State<AppState>, Path(slug): Path<String>) -> Response {
  if validate_slug(&slug).is_err() {
    return not_found_page();
  }
  let target = match state.cache.get(&slug).await {
    Some(cached) => cached,
    None => {
      let row = links::Entity::find()
        .filter(links::Column::Slug.eq(&slug))
        .one(&state.db)
        .await;
      let resolved = match row {
        Ok(Some(row)) if row.enabled => CachedLink::Url(row.url),
        Ok(_) => CachedLink::Missing,
        Err(e) => {
          eprintln!("redirect lookup failed for '{slug}': {e}");
          return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response();
        }
      };
      state.cache.put(&slug, &resolved).await;
      resolved
    }
  };
  match target {
    CachedLink::Url(url) => found(&url),
    CachedLink::Missing => not_found_page(),
  }
}

/// `302 Found` — links are editable, so browsers must not cache the hop
/// permanently (no 301/308).
fn found(url: &str) -> Response {
  match HeaderValue::from_str(url) {
    Ok(location) => (
      StatusCode::FOUND,
      [
        (header::LOCATION, location),
        (header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=0")),
      ],
    )
      .into_response(),
    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Invalid redirect target").into_response(),
  }
}

const NOT_FOUND_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<meta name="robots" content="noindex" />
<title>Link not found</title>
<style>
  :root { color-scheme: light dark; --bg: #faf9f5; --fg: #3d3929; --muted: #83827d; --accent: #c96442; }
  @media (prefers-color-scheme: dark) { :root { --bg: #262624; --fg: #c3c0b6; --muted: #b7b5a9; --accent: #d97757; } }
  * { box-sizing: border-box; }
  body { margin: 0; min-height: 100svh; display: grid; place-items: center; background: var(--bg); color: var(--fg);
    font-family: ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif; padding: 24px; }
  main { text-align: center; max-width: 28rem; }
  .code { font-size: 4.5rem; font-weight: 700; letter-spacing: -0.04em; color: var(--accent); line-height: 1; margin: 0; }
  h1 { font-size: 1.125rem; margin: 1rem 0 0.5rem; }
  p { margin: 0; color: var(--muted); font-size: 0.925rem; line-height: 1.5; }
</style>
</head>
<body>
<main>
  <p class="code">404</p>
  <h1>This link doesn't exist</h1>
  <p>It may have been removed, disabled, or mistyped. Check the address and try again.</p>
</main>
</body>
</html>
"#;

/// HTML 404 for unknown / disabled slugs and any other non-API path.
pub fn not_found_page() -> Response {
  (
    StatusCode::NOT_FOUND,
    [(header::CACHE_CONTROL, "public, max-age=60")],
    Html(NOT_FOUND_HTML),
  )
    .into_response()
}
