import { boolean, index, pgTable, text, timestamp, varchar } from 'drizzle-orm/pg-core';

/**
 * Short links. `id` is a stable **nanoid** generated in application code (no DB
 * default, see Rust `utils::id`); `slug` is the public, editable path segment
 * (`/<slug>`), unique and case-sensitive.
 */
export const links = pgTable(
  'links',
  {
    /** nanoid — generated in application code */
    id: text().primaryKey(),
    /** Public path segment: `[A-Za-z0-9_-]{1,64}` (validated in the API) */
    slug: varchar({ length: 64 }).notNull().unique(),
    /** Redirect target (absolute http(s) URL) */
    url: text().notNull(),
    /** Optional human label */
    title: varchar({ length: 255 }),
    /** Disabled links answer 404 instead of redirecting */
    enabled: boolean().default(true).notNull(),
    /** Better Auth user id of the creator */
    createdBy: text(),
    createdAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
    updatedAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [index('links_createdAt_idx').on(table.createdAt)],
);
