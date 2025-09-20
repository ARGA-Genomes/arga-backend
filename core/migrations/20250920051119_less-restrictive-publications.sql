-- Modify "publications" table
ALTER TABLE "public"."publications" ALTER COLUMN "title" DROP NOT NULL, ALTER COLUMN "authors" DROP NOT NULL, ALTER COLUMN "published_year" DROP NOT NULL;
