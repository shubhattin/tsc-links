-- Well-known expense sources. Run after migrations against either database:
--   psql "$PG_DATABASE_URL"  -f scripts/seed-expense-sources.sql
--   psql "$PG_DATABASE_URL1" -f scripts/seed-expense-sources.sql
-- Idempotent (re-run updates name/description in place).
INSERT INTO expense_sources (key, name, description) VALUES
  ('vercel', 'Vercel', 'Hosting, serverless functions and bandwidth'),
  ('gcp', 'Google Cloud', 'GCP projects: compute, storage and APIs'),
  ('aws', 'AWS', 'Amazon Web Services: compute, storage and CDN'),
  ('cloudflare', 'Cloudflare', 'DNS, CDN, Workers and security'),
  ('upstash', 'Upstash', 'Serverless Redis and Kafka'),
  ('neon', 'Neon Database', 'Serverless Postgres'),
  ('misc', 'Miscellaneous', 'One-off costs that fit nowhere else'),
  ('openrouter', 'OpenRouter', 'Unified LLM API gateway'),
  ('openai-platform', 'OpenAI Platform', 'OpenAI API: models and embeddings')
ON CONFLICT (key) DO UPDATE SET
  name = EXCLUDED.name,
  description = EXCLUDED.description,
  "updatedAt" = now();
