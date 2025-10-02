-- Modify "assemblies" table
ALTER TABLE "public"."assemblies" DROP COLUMN "genome_coverage", ADD COLUMN "level" character varying NULL, ADD COLUMN "size_ungapped" bigint NULL, ADD COLUMN "guanine_cytosine_percent" double precision NULL, ADD COLUMN "coverage" character varying NULL, ADD COLUMN "representation" character varying NULL, ADD COLUMN "number_of_contigs" integer NULL, ADD COLUMN "number_of_replicons" integer NULL;


-- manually change type
ALTER TABLE assemblies DROP COLUMN "size", DROP COLUMN "minimum_gap_length", DROP COLUMN "number_of_scaffolds";
ALTER TABLE assemblies ADD COLUMN "size" bigint, ADD COLUMN "minimum_gap_length" bigint, ADD COLUMN "number_of_scaffolds" integer;
