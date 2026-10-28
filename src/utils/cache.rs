//! Redis cache for the redirect hot path: `slug → target URL`.
//!
//! - Hit: `tsc-links:slug:<slug>` holds the target URL ([`HIT_TTL_SECS`]).
//! - Negative entry: an empty string marks a missing/disabled slug
//!   ([`MISS_TTL_SECS`]) so unknown paths (bots, typos) don't hammer Postgres.
//!
//! Admin writes go through the cache (create/update set the new state,
//! slug renames and deletes drop the old key). Every Redis error is logged and
//! treated as a miss — Postgres stays the source of truth, so a Redis outage
//! only costs latency.

use redis::AsyncCommands;
use redis::aio::{ConnectionManager, ConnectionManagerConfig};
use std::time::Duration;

const KEY_PREFIX: &str = "tsc-links:slug:";
/// Positive entries; bounded so a lost write-race heals itself.
const HIT_TTL_SECS: u64 = 60 * 60;
/// Negative entries (slug missing or disabled).
const MISS_TTL_SECS: u64 = 60;
const MISS_MARKER: &str = "";

/// What the cache knows about a slug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CachedLink {
  /// Enabled link → redirect target.
  Url(String),
  /// Known to be missing or disabled.
  Missing,
}

/// Cheap-to-clone handle; `None` inside means caching is disabled.
#[derive(Clone)]
pub struct LinkCache {
  conn: Option<ConnectionManager>,
}

fn key(slug: &str) -> String {
  format!("{KEY_PREFIX}{slug}")
}

impl LinkCache {
  /// Cache that never stores anything.
  pub fn disabled() -> Self {
    Self { conn: None }
  }

  /// Connect to Redis; on any failure log and fall back to [`Self::disabled`].
  pub async fn connect(url: Option<&str>) -> Self {
    let Some(url) = url else {
      eprintln!("REDIS_URL not set: link cache disabled");
      return Self::disabled();
    };
    let client = match redis::Client::open(url) {
      Ok(c) => c,
      Err(e) => {
        eprintln!("invalid REDIS_URL, link cache disabled: {e}");
        return Self::disabled();
      }
    };
    // Tight timeouts: a slow Redis must never stall a redirect for long.
    let config = ConnectionManagerConfig::new()
      .set_connection_timeout(Some(Duration::from_secs(3)))
      .set_response_timeout(Some(Duration::from_millis(800)))
      .set_number_of_retries(1);
    match client.get_connection_manager_with_config(config).await {
      Ok(conn) => Self { conn: Some(conn) },
      Err(e) => {
        eprintln!("failed to connect to Redis, link cache disabled: {e}");
        Self::disabled()
      }
    }
  }

  /// `None` when not cached (or Redis is unavailable).
  pub async fn get(&self, slug: &str) -> Option<CachedLink> {
    let mut conn = self.conn.clone()?;
    match conn.get::<_, Option<String>>(key(slug)).await {
      Ok(Some(v)) if v == MISS_MARKER => Some(CachedLink::Missing),
      Ok(Some(url)) => Some(CachedLink::Url(url)),
      Ok(None) => None,
      Err(e) => {
        eprintln!("redis GET failed: {e}");
        None
      }
    }
  }

  /// Store the redirect state for a slug.
  pub async fn put(&self, slug: &str, value: &CachedLink) {
    let Some(mut conn) = self.conn.clone() else {
      return;
    };
    let (val, ttl) = match value {
      CachedLink::Url(url) => (url.as_str(), HIT_TTL_SECS),
      CachedLink::Missing => (MISS_MARKER, MISS_TTL_SECS),
    };
    if let Err(e) = conn.set_ex::<_, _, ()>(key(slug), val, ttl).await {
      eprintln!("redis SET failed: {e}");
    }
  }

  /// Drop cached state for a slug (next redirect re-reads Postgres).
  pub async fn invalidate(&self, slug: &str) {
    let Some(mut conn) = self.conn.clone() else {
      return;
    };
    if let Err(e) = conn.del::<_, ()>(key(slug)).await {
      eprintln!("redis DEL failed: {e}");
    }
  }
}
