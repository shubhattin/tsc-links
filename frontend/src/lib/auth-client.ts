import { PUBLIC_BETTER_AUTH_URL } from '$env/static/public';
import { adminClient } from 'better-auth/client/plugins';
import { createAuthClient } from 'better-auth/svelte';

/**
 * Better Auth client talking directly to the tsc-users auth server. Session
 * cookies are shared across subdomains, so the Rust API sees the same session.
 */
export const authClient = createAuthClient({
  baseURL: PUBLIC_BETTER_AUTH_URL,
  plugins: [adminClient()],
});

export type Session = typeof authClient.$Infer.Session;

/** Better Auth's admin plugin stores roles as a comma-separated string. */
export function isAdmin(user: { role?: string | null } | null | undefined): boolean {
  return (user?.role ?? '').split(',').some((r) => r.trim() === 'admin');
}

export async function signInWith(provider: 'google' | 'github') {
  const { error } = await authClient.signIn.social({
    provider,
    // Absolute: the auth server lives on another origin.
    callbackURL: window.location.href,
  });
  if (error) throw new Error(error.message ?? 'Sign-in failed');
}
