-- Modify "assemblies" table
ALTER TABLE "public"."assemblies" ALTER COLUMN "total_contig_size" TYPE bigint, ALTER COLUMN "total_scaffold_size" TYPE bigint;
