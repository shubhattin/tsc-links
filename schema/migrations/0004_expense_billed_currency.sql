ALTER TABLE "expenses" RENAME COLUMN "usdCents" TO "billedCents";--> statement-breakpoint
ALTER TABLE "expenses" ADD COLUMN "billedCurrency" varchar(3) DEFAULT 'USD' NOT NULL;