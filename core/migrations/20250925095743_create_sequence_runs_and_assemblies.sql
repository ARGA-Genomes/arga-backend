-- Create "assemblies" table
CREATE TABLE "public"."assemblies" (
 "entity_id" character varying NOT NULL,
 "species_name_id" bigint NOT NULL,
 "publication_id" character varying NULL,
 "assembly_id" character varying NOT NULL,
 "event_date" date NULL,
 "event_time" time NULL,
 "name" character varying NULL,
 "type" character varying NULL,
 "method" character varying NULL,
 "method_version" character varying NULL,
 "method_link" character varying NULL,
 "size" character varying NULL,
 "minimum_gap_length" character varying NULL,
 "completeness" character varying NULL,
 "completeness_method" character varying NULL,
 "source_molecule" character varying NULL,
 "reference_genome_used" character varying NULL,
 "reference_genome_link" character varying NULL,
 "number_of_scaffolds" character varying NULL,
 "genome_coverage" character varying NULL,
 "hybrid" character varying NULL,
 "hybrid_information" character varying NULL,
 "polishing_or_scaffolding_method" character varying NULL,
 "polishing_or_scaffolding_data" character varying NULL,
 "computational_infrastructure" character varying NULL,
 "system_used" character varying NULL,
 "assembly_n50" character varying NULL,
 PRIMARY KEY ("entity_id"),
 CONSTRAINT "assemblies_publication_id_fkey" FOREIGN KEY ("publication_id") REFERENCES "public"."publications" ("entity_id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "assembly_logs" table
CREATE TABLE "public"."assembly_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "assembly_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "assembly_logs_dataset_version_id" to table: "assembly_logs"
CREATE INDEX "assembly_logs_dataset_version_id" ON "public"."assembly_logs" ("dataset_version_id");
-- Create index "assembly_logs_entity_id" to table: "assembly_logs"
CREATE INDEX "assembly_logs_entity_id" ON "public"."assembly_logs" ("entity_id");
-- Create index "assembly_logs_parent_id" to table: "assembly_logs"
CREATE INDEX "assembly_logs_parent_id" ON "public"."assembly_logs" ("parent_id");
-- Create "sequence_run_logs" table
CREATE TABLE "public"."sequence_run_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "sequence_run_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "sequence_run_logs_dataset_version_id" to table: "sequence_run_logs"
CREATE INDEX "sequence_run_logs_dataset_version_id" ON "public"."sequence_run_logs" ("dataset_version_id");
-- Create index "sequence_run_logs_entity_id" to table: "sequence_run_logs"
CREATE INDEX "sequence_run_logs_entity_id" ON "public"."sequence_run_logs" ("entity_id");
-- Create index "sequence_run_logs_parent_id" to table: "sequence_run_logs"
CREATE INDEX "sequence_run_logs_parent_id" ON "public"."sequence_run_logs" ("parent_id");
-- Create "sequence_runs" table
CREATE TABLE "public"."sequence_runs" (
 "entity_id" character varying NOT NULL,
 "library_id" character varying NOT NULL,
 "species_name_id" bigint NOT NULL,
 "publication_id" character varying NULL,
 "sequence_run_id" character varying NOT NULL,
 "event_date" date NULL,
 "event_time" time NULL,
 "facility" character varying NULL,
 "instrument_or_method" character varying NULL,
 "platform" character varying NULL,
 "kit_chemistry" character varying NULL,
 "flowcell_type" character varying NULL,
 "cell_movie_length" character varying NULL,
 "base_caller_model" character varying NULL,
 "fast5_compression" character varying NULL,
 "analysis_software" character varying NULL,
 "analysis_software_version" character varying NULL,
 "target_gene" character varying NULL,
 "sra_run_accession" character varying NULL,
 PRIMARY KEY ("entity_id"),
 CONSTRAINT "sequence_runs_library_id_fkey" FOREIGN KEY ("library_id") REFERENCES "public"."libraries" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "sequence_runs_publication_id_fkey" FOREIGN KEY ("publication_id") REFERENCES "public"."publications" ("entity_id") ON UPDATE NO ACTION ON DELETE NO ACTION
);



-- manually added entity views
CREATE MATERIALIZED VIEW sequence_run_entities AS
SELECT entity_id FROM sequence_run_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX sequence_run_entities_entity_id ON sequence_run_entities (entity_id);

CREATE MATERIALIZED VIEW assembly_entities AS
SELECT entity_id FROM assembly_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX assembly_entities_entity_id ON assembly_entities (entity_id);
