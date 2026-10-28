//! Environment-driven configuration held in [`AppState`](crate::state::AppState).
//!
//! `PUBLIC_BETTER_AUTH_URL` is required (admin API auth). `REDIS_URL` is
//! optional: without it redirects read Postgres directly.

#[derive(Clone, Debug)]
pub struct EnvConfig {
  /// Better Auth server origin (`PUBLIC_BETTER_AUTH_URL`, no trailing slash).
  /// Shared with the frontend's auth client.
  pub better_auth_url: String,
  /// Redis connection URL (`REDIS_URL`, `redis://` or `rediss://`).
  pub redis_url: Option<String>,
}

fn required_var(name: &str) -> Result<String, String> {
  match std::env::var(name) {
    Ok(v) if !v.trim().is_empty() => Ok(v.trim().to_string()),
    _ => Err(format!("{name} must be set: missing environment variable")),
  }
}

fn optional_var(name: &str) -> Option<String> {
  std::env::var(name)
    .ok()
    .map(|v| v.trim().to_string())
    .filter(|v| !v.is_empty())
}

impl EnvConfig {
  pub fn from_env() -> Result<Self, String> {
    let better_auth_url = required_var("PUBLIC_BETTER_AUTH_URL")?
      .trim_end_matches('/')
      .to_string();
    if !better_auth_url.starts_with("http://") && !better_auth_url.starts_with("https://") {
      return Err("PUBLIC_BETTER_AUTH_URL must be set: expected an http(s) URL".to_string());
    }
    Ok(Self {
      better_auth_url,
      redis_url: optional_var("REDIS_URL"),
    })
  }
}
