CREATE TABLE "links" (
	"id" text PRIMARY KEY NOT NULL,
	"slug" varchar(64) NOT NULL,
	"url" text NOT NULL,
	"title" varchar(255),
	"enabled" boolean DEFAULT true NOT NULL,
	"createdBy" text,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	"updatedAt" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "links_slug_unique" UNIQUE("slug")
);
--> statement-breakpoint
CREATE INDEX "links_createdAt_idx" ON "links" USING btree ("createdAt");