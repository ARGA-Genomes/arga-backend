-- Create "data_products" table
CREATE TABLE "public"."data_products" (
 "entity_id" character varying NOT NULL,
 "publication_id" character varying NULL,
 "organism_id" character varying NULL,
 "extract_id" character varying NULL,
 "sequence_run_id" character varying NULL,
 "custodian" character varying NULL,
 "sequence_sample_id" character varying NULL,
 "sequence_analysis_id" character varying NULL,
 "notes" character varying NULL,
 "context" character varying NULL,
 "type" character varying NULL,
 "file_type" character varying NULL,
 "url" character varying NULL,
 "licence" character varying NULL,
 "access" character varying NULL,
 PRIMARY KEY ("entity_id"),
 CONSTRAINT "data_products_custodian_fkey" FOREIGN KEY ("custodian") REFERENCES "public"."agents" ("entity_id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "data_products_extract_id_fkey" FOREIGN KEY ("extract_id") REFERENCES "public"."dna_extracts" ("entity_id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "data_products_organism_id_fkey" FOREIGN KEY ("organism_id") REFERENCES "public"."organisms" ("entity_id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "data_products_publication_id_fkey" FOREIGN KEY ("publication_id") REFERENCES "public"."publications" ("entity_id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "data_products_sequence_run_id_fkey" FOREIGN KEY ("sequence_run_id") REFERENCES "public"."sequence_runs" ("entity_id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
