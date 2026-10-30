CREATE TABLE "expense_sources" (
	"key" varchar(32) PRIMARY KEY NOT NULL,
	"name" varchar(64) NOT NULL,
	"description" text,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	"updatedAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "expenses" (
	"id" bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY (sequence name "expenses_id_seq" INCREMENT BY 1 MINVALUE 1 MAXVALUE 9223372036854775807 START WITH 1 CACHE 1),
	"sourceKey" varchar(32) NOT NULL,
	"date" date NOT NULL,
	"label" varchar(120),
	"usdCents" bigint,
	"inrPaise" bigint NOT NULL,
	"invoiceLink" text,
	"invoiceBlob" "bytea",
	"invoiceMime" varchar(64),
	"invoiceName" varchar(255),
	"createdBy" text,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	"updatedAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "expenses" ADD CONSTRAINT "expenses_sourceKey_expense_sources_key_fk" FOREIGN KEY ("sourceKey") REFERENCES "public"."expense_sources"("key") ON DELETE restrict ON UPDATE no action;--> statement-breakpoint
CREATE INDEX "expenses_sourceKey_date_idx" ON "expenses" USING btree ("sourceKey","date");--> statement-breakpoint
CREATE INDEX "expenses_date_idx" ON "expenses" USING btree ("date");