import {
  bigint,
  boolean,
  customType,
  date,
  index,
  pgTable,
  text,
  timestamp,
  varchar,
} from 'drizzle-orm/pg-core';

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

/** Raw Postgres `bytea` (invoice uploads). */
const bytea = customType<{ data: Buffer; driverData: string }>({
  dataType: () => 'bytea',
});

/**
 * Where money can be spent. `key` is a short lowercase id (`vercel`, `neon`,
 * …) and the primary key; seeded with the well-known vendors (see
 * `scripts/seed-expense-sources.sql`).
 */
export const expenseSources = pgTable('expense_sources', {
  /** Lowercase source id, e.g. `vercel` */
  key: varchar({ length: 32 }).primaryKey(),
  /** Display name, e.g. `Vercel` */
  name: varchar({ length: 64 }).notNull(),
  /** Optional one-liner */
  description: text(),
  createdAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
  updatedAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
});

/**
 * One charged cost. A source can have many rows per month (the UI aggregates
 * them). Money is integer minor units (paise/cents) — no float rounding.
 */
export const expenses = pgTable(
  'expenses',
  {
    /** bigserial — database-generated */
    id: bigint({ mode: 'number' }).primaryKey().generatedAlwaysAsIdentity(),
    /** FK → `expense_sources.key` (`ON DELETE RESTRICT`) */
    sourceKey: varchar({ length: 32 })
      .notNull()
      .references(() => expenseSources.key, { onDelete: 'restrict' }),
    /** Day precision (`YYYY-MM-DD`) */
    date: date().notNull(),
    /** Optional pill label, e.g. `Pro plan` */
    label: varchar({ length: 120 }),
    /** Optional short explanatory note */
    note: text(),
    /** Optional charged USD amount, in cents */
    usdCents: bigint({ mode: 'number' }),
    /** Final charged INR amount, in paise (required) */
    inrPaise: bigint({ mode: 'number' }).notNull(),
    /** Optional external invoice URL */
    invoiceLink: text(),
    /** Optional raw invoice bytes (images re-encoded to webp on upload) */
    invoiceBlob: bytea(),
    invoiceMime: varchar({ length: 64 }),
    invoiceName: varchar({ length: 255 }),
    /** Better Auth user id of the creator */
    createdBy: text(),
    createdAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
    updatedAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
  },
  (table) => [
    index('expenses_sourceKey_date_idx').on(table.sourceKey, table.date),
    index('expenses_date_idx').on(table.date),
  ],
);
