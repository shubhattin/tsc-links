//! Public short-link slugs (`/<slug>`).
//!
//! Slugs are case-sensitive, 1–64 chars of `[A-Za-z0-9_-]`. They never contain
//! a `.` or `/`, so they can't shadow static files (`/favicon.ico`,
//! `/robots.txt`) or nested paths (`/_app/…`). Generated slugs are short
//! alphanumeric nanoids (no `-`/`_`, so they read cleanly in a URL).

use nanoid::nanoid;

pub const SLUG_MAX_LEN: usize = 64;

/// Length of auto-generated slugs. 62^7 ≈ 3.5e12 — collisions are retried.
pub const GENERATED_SLUG_LEN: usize = 7;

const ALPHABET: [char; 62] = [
  '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
  'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
  'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
  'V', 'W', 'X', 'Y', 'Z',
];

/// Path segments owned by the app itself (compared case-insensitively).
const RESERVED: &[&str] = &["api", "_app"];

/// Generate a random slug (not checked for uniqueness).
pub fn generate_slug() -> String {
  nanoid!(GENERATED_SLUG_LEN, &ALPHABET)
}

/// Charset + length check only (used on the redirect hot path).
pub fn is_well_formed(slug: &str) -> bool {
  !slug.is_empty()
    && slug.len() <= SLUG_MAX_LEN
    && slug
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Full validation for user-chosen slugs; returns the error message on failure.
pub fn validate_slug(slug: &str) -> Result<(), String> {
  if !is_well_formed(slug) {
    return Err(format!(
      "slug must be 1-{SLUG_MAX_LEN} characters of letters, digits, '-' or '_'"
    ));
  }
  if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(slug)) {
    return Err(format!("slug '{slug}' is reserved"));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn generated_slugs_are_valid() {
    for _ in 0..100 {
      let s = generate_slug();
      assert_eq!(s.len(), GENERATED_SLUG_LEN);
      assert!(validate_slug(&s).is_ok());
    }
  }

  #[test]
  fn rejects_bad_slugs() {
    for bad in ["", "a/b", "favicon.ico", "with space", "API", "_app", "é"] {
      assert!(validate_slug(bad).is_err(), "{bad} should be rejected");
    }
    assert!(validate_slug(&"x".repeat(SLUG_MAX_LEN + 1)).is_err());
    assert!(validate_slug("my-Link_2").is_ok());
  }
}
