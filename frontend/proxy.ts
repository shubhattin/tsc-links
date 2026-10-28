import type { ProxyOptions } from 'vite';

// The axum server listens here in local dev (`cargo run --bin main`).
// Override with AXUM_DEV_ORIGIN=http://127.0.0.1:<port> if needed.
const AXUM_DEV_PORT = 5779;
const AXUM_DEV_TARGET = process.env.AXUM_DEV_ORIGIN ?? `http://127.0.0.1:${AXUM_DEV_PORT}`;

// Mirror the root vercel.json during local dev: SvelteKit owns `/` and its
// own assets; `/api/*` and single-segment slugs (`/abc123`) go to axum.
// Vite internals (`/@vite/…`, `/src/…`, `/node_modules/…`) and files with an
// extension never match the slug pattern, so they stay with Vite.
const axumProxy: ProxyOptions = {
  target: AXUM_DEV_TARGET,
  changeOrigin: true,
  secure: false,
};

export const proxy: Record<string, ProxyOptions> = {
  '^/api(?:/.*)?$': axumProxy,
  '^/[A-Za-z0-9_-]+/?$': axumProxy,
};
