import ky from 'ky';

/**
 * Shared ky instance. Same-origin (Vite proxy in dev, Vercel rewrite in prod),
 * so the browser attaches the Better Auth session cookie on its own — the Rust
 * API forwards it to the auth server's `get-session`.
 */
export const api = ky.create({
  credentials: 'same-origin',
  timeout: 30_000,
  retry: 0,
});
