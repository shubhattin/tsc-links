//! Application-generated row IDs (**nanoid**).
//!
//! `links.id` is `text` in Postgres (not UUID). Call [`new_id`] when inserting
//! rows — there is no DB-side default. Public slugs live in [`super::slug`].

use nanoid::nanoid;

/// Default nanoid length (URL-safe alphabet).
pub const ID_LENGTH: usize = 21;

/// Generate a new nanoid string for `links.id`.
pub fn new_id() -> String {
  nanoid!(ID_LENGTH)
}
