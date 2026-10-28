/**
 * App-facing links API: thin typed wrappers over the generated Hey API client.
 */
import { client } from '$lib/api/index.js';
import type { CreateLinkBody, ErrorBody, LinkDto, UpdateLinkBody } from '$lib/api/index.js';

export type { CreateLinkBody, LinkDto, UpdateLinkBody };

export const LINKS_QUERY_KEY = ['links'] as const;

function errorMessage(error: unknown, status: number | undefined, fallback: string): string {
  if (error && typeof error === 'object') {
    const msg = (error as Partial<ErrorBody>).error;
    if (typeof msg === 'string' && msg.length > 0) return msg;
  }
  return status ? `${fallback} (HTTP ${status})` : fallback;
}

async function unwrap<T>(
  promise: Promise<{ data?: T; error?: unknown; response?: Response }>,
  fallback: string,
): Promise<T> {
  const { data, error, response } = await promise;
  if (error || data === undefined) throw new Error(errorMessage(error, response?.status, fallback));
  return data;
}

export async function listLinks(): Promise<LinkDto[]> {
  const res = await unwrap(client.links.list(), 'Could not load links');
  return res.links;
}

export function createLink(body: CreateLinkBody): Promise<LinkDto> {
  return unwrap(client.links.create({ body }), 'Could not create link');
}

export function updateLink(id: string, body: UpdateLinkBody): Promise<LinkDto> {
  return unwrap(client.links.update({ path: { id }, body }), 'Could not update link');
}

export async function deleteLink(id: string): Promise<void> {
  const { error, response } = await client.links.delete({ path: { id } });
  if (error || !response?.ok) {
    throw new Error(errorMessage(error, response?.status, 'Could not delete link'));
  }
}

/** Public short URL for a slug (the app and its redirects share one origin). */
export function shortUrl(slug: string): string {
  return `${window.location.origin}/${slug}`;
}

/** `example.com/path` display form of a short link (no scheme). */
export function shortUrlDisplay(slug: string): string {
  return `${window.location.host}/${slug}`;
}

/** Mirrors the Rust validator (`utils::slug`). */
export const SLUG_PATTERN = /^[A-Za-z0-9_-]{1,64}$/;
const RESERVED_SLUGS = ['api', '_app'];

export function slugError(slug: string): string | null {
  if (!slug) return null;
  if (!SLUG_PATTERN.test(slug)) return "Use 1–64 letters, digits, '-' or '_'";
  if (RESERVED_SLUGS.includes(slug.toLowerCase())) return `'${slug}' is reserved`;
  return null;
}

const relativeFormat = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });
const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ['year', 365 * 24 * 3600],
  ['month', 30 * 24 * 3600],
  ['week', 7 * 24 * 3600],
  ['day', 24 * 3600],
  ['hour', 3600],
  ['minute', 60],
];

export function timeAgo(iso: string): string {
  const seconds = (new Date(iso).getTime() - Date.now()) / 1000;
  for (const [unit, size] of UNITS) {
    if (Math.abs(seconds) >= size) return relativeFormat.format(Math.round(seconds / size), unit);
  }
  return 'just now';
}
