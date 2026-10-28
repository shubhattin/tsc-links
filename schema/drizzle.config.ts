import type { Config } from 'drizzle-kit';
import dotenv from 'dotenv';

dotenv.config({ path: '../.env.local' });

const DB_URL =
  process.env.DB_MODE === 'PROD' ? process.env.PG_DATABASE_URL1 : process.env.PG_DATABASE_URL;

export default {
  schema: './schema.ts',
  out: './migrations',
  dialect: 'postgresql',
  dbCredentials: {
    url: DB_URL!,
  },
} satisfies Config;
