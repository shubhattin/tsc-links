//! Shared helpers: env config, database pool, Redis link cache, id / slug
//! generation and validation.
//!
//! App wiring (`app`, `state`, `auth`, `openapi`, `routes`, `entities`) lives
//! at the crate root; everything here is supporting infrastructure.

pub mod cache;
pub mod config;
pub mod db;
pub mod id;
pub mod slug;
