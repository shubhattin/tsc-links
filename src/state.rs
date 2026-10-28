//! Shared application state (extractable via `State<AppState>`).
//!
//! - `db`: SeaORM Postgres pool.
//! - `cache`: Redis slug → URL cache (no-op when `REDIS_URL` is unset/unreachable).
//! - `http`: shared HTTP client (Better Auth session lookups).
//! - `config`: validated env config ([`crate::utils::config::EnvConfig`]).

use crate::utils::cache::LinkCache;
use crate::utils::config::EnvConfig;
use crate::utils::db::DbPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
  pub db: DbPool,
  pub cache: LinkCache,
  pub http: reqwest::Client,
  pub config: Arc<EnvConfig>,
}
