-- Modify "projects" table
ALTER TABLE "public"."projects" ALTER COLUMN "data_context" TYPE text[], ALTER COLUMN "data_types" TYPE text[], ALTER COLUMN "data_assay_types" TYPE text[], ALTER COLUMN "partners" TYPE text[];
