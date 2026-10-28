//! OpenAPI document composition (paths collected via [`utoipa_axum::OpenApiRouter`] in route modules).

use utoipa::Modify;
use utoipa::OpenApi;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

use crate::auth::ErrorBody;
use crate::routes::links::{CreateLinkBody, LinkDto, LinksListResponse, UpdateLinkBody};

/// Root OpenAPI document; operation paths are registered by nested [`utoipa_axum::OpenApiRouter`]s.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "TSC Links API",
        version = "0.1.0",
        description = "Short-link admin API. Redirects (`GET /{slug}`) are public and not part of this document."
    ),
    components(schemas(ErrorBody, LinkDto, LinksListResponse, CreateLinkBody, UpdateLinkBody)),
    modifiers(&SecurityAddon),
    tags((name = "links", description = "Short links (Better Auth admin session required)"))
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    if let Some(components) = openapi.components.as_mut() {
      components.add_security_scheme(
        "session_cookie",
        SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(
          "better-auth.session_token",
        ))),
      );
    }
  }
}

/// Build the merged OpenAPI spec (used by `gen-openapi`).
pub fn openapi() -> utoipa::openapi::OpenApi {
  let (_, api) = crate::app::openapi_router().split_for_parts();
  api
}
