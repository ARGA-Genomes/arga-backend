CREATE SCHEMA IF NOT EXISTS "public";

CREATE EXTENSION IF NOT EXISTS postgis SCHEMA "public";

-- Create enum type "source_content_type"
CREATE TYPE "public"."source_content_type" AS ENUM ('taxonomic_backbone', 'ecological_traits', 'genomic_data', 'specimens', 'nongenomic_data', 'morphological_traits', 'biochemical_traits', 'mixed_datatypes', 'functional_traits', 'ethnobiology');
-- Create "imcra_provincial" table
CREATE TABLE "public"."imcra_provincial" (
 "ogc_fid" integer NOT NULL,
 "pb_name" character varying NULL,
 "pb_num" integer NULL,
 "water_type" character varying NULL,
 "area_km2" double precision NULL,
 "wkb_geometry" public.geometry(MultiPolygon,4283) NULL,
 PRIMARY KEY ("ogc_fid")
);
-- Create index "imcra_provincial_wkb_geometry_geom_idx" to table: "imcra_provincial"
CREATE INDEX "imcra_provincial_wkb_geometry_geom_idx" ON "public"."imcra_provincial" USING gist ("wkb_geometry");
-- Create enum type "taxonomic_status"
CREATE TYPE "public"."taxonomic_status" AS ENUM ('accepted', 'undescribed', 'species_inquirenda', 'manuscript_name', 'hybrid', 'synonym', 'unaccepted', 'informal', 'placeholder', 'basionym', 'nomenclatural_synonym', 'taxonomic_synonym', 'replaced_synonym', 'orthographic_variant', 'misapplied', 'excluded', 'alternative_name', 'pro_parte_misapplied', 'pro_parte_taxonomic_synonym', 'doubtful_misapplied', 'doubtful_taxonomic_synonym', 'doubtful_pro_parte_misapplied', 'doubtful_pro_parte_taxonomic_synonym', 'taxon_inquirendum', 'homonym', 'misspelled', 'unassessed', 'unavailable', 'uncertain', 'unjustified_emendation', 'nomen_dubium', 'nomen_nudum', 'nomen_oblitum', 'interim_unpublished', 'incorrect_grammatical_agreement_of_specific_epithet', 'superseded_combination', 'superseded_rank');
-- Create enum type "taxonomic_rank"
CREATE TYPE "public"."taxonomic_rank" AS ENUM ('domain', 'superkingdom', 'kingdom', 'subkingdom', 'phylum', 'subphylum', 'superclass', 'class', 'subclass', 'superorder', 'order', 'suborder', 'hyporder', 'minorder', 'superfamily', 'family', 'subfamily', 'supertribe', 'tribe', 'subtribe', 'genus', 'subgenus', 'higher_taxon', 'regnum', 'familia', 'classis', 'ordo', 'varietas', 'forma', 'subclassis', 'superordo', 'sectio', 'nothovarietas', 'subvarietas', 'series', 'subfamilia', 'subordo', 'regio', 'species', 'subspecies', 'infraspecies', 'aggregate_genera', 'aggregate_species', 'cohort', 'subcohort', 'division', 'infraclass', 'infraorder', 'section', 'subdivision', 'incertae_sedis', 'special_form', 'unranked', 'subsectio', 'superspecies', 'infragenus', 'subforma', 'subseries', 'infrakingdom', 'superphylum', 'infraphylum', 'parvphylum', 'gigaclass', 'megaclass', 'subterclass', 'parvorder', 'epifamily', 'variety', 'subvariety', 'natio', 'mutatio', 'subsection', 'pathovar', 'serovar', 'biovar', 'supercohort');
-- Create enum type "job_status"
CREATE TYPE "public"."job_status" AS ENUM ('pending', 'initialized', 'running', 'completed', 'failed', 'dead');
-- Create enum type "region_type"
CREATE TYPE "public"."region_type" AS ENUM ('ibra', 'imcra', 'state', 'drainage_basin');
-- Create enum type "operation_action"
CREATE TYPE "public"."operation_action" AS ENUM ('create', 'update', 'delete');
-- Create enum type "data_reuse_status"
CREATE TYPE "public"."data_reuse_status" AS ENUM ('limited', 'unlimited', 'none', 'variable');
-- Create enum type "access_rights_status"
CREATE TYPE "public"."access_rights_status" AS ENUM ('open', 'restricted', 'conditional', 'variable');
-- Create "sources" table
CREATE TABLE "public"."sources" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "name" character varying NOT NULL,
 "author" character varying NOT NULL,
 "rights_holder" character varying NOT NULL,
 "access_rights" character varying NOT NULL,
 "license" character varying NOT NULL,
 "reuse_pill" "public"."data_reuse_status" NULL,
 "access_pill" "public"."access_rights_status" NULL,
 "content_type" "public"."source_content_type" NULL,
 "lists_id" character varying(24) NULL,
 PRIMARY KEY ("id")
);
-- Create index "sources_name" to table: "sources"
CREATE UNIQUE INDEX "sources_name" ON "public"."sources" ("name");
-- Set comment to table: "sources"
COMMENT ON TABLE "public"."sources" IS 'Information and metadata about imported data';
-- Create "ibra" table
CREATE TABLE "public"."ibra" (
 "ogc_fid" integer NOT NULL,
 "reg_code_7" character varying NULL,
 "reg_name_7" character varying NULL,
 "hectares" double precision NULL,
 "sq_km" double precision NULL,
 "rec_id" integer NULL,
 "reg_code_6" character varying NULL,
 "reg_name_6" character varying NULL,
 "reg_no_61" double precision NULL,
 "feat_id" character varying NULL,
 "shape_leng" double precision NULL,
 "shape_area" double precision NULL,
 "wkb_geometry" public.geometry(MultiPolygon,4283) NULL,
 PRIMARY KEY ("ogc_fid")
);
-- Create index "ibra_wkb_geometry_geom_idx" to table: "ibra"
CREATE INDEX "ibra_wkb_geometry_geom_idx" ON "public"."ibra" USING gist ("wkb_geometry");
-- Create enum type "nomenclatural_act_type"
CREATE TYPE "public"."nomenclatural_act_type" AS ENUM ('species_nova', 'subspecies_nova', 'genus_species_nova', 'combinatio_nova', 'revived_status', 'name_usage', 'original_description', 'redescription', 'demotion', 'promotion', 'synonymisation', 'heterotypic_synonymy', 'homotypic_synonymy');
-- Create enum type "publication_type"
CREATE TYPE "public"."publication_type" AS ENUM ('book', 'book_chapter', 'journal_article', 'journal_volume', 'proceedings_paper', 'url');
-- Create "imcra_mesoscale" table
CREATE TABLE "public"."imcra_mesoscale" (
 "ogc_fid" integer NOT NULL,
 "meso_name" character varying NULL,
 "meso_num" integer NULL,
 "meso_abbr" character varying NULL,
 "water_type" character varying NULL,
 "area_km2" double precision NULL,
 "wkb_geometry" public.geometry(MultiPolygon,4283) NULL,
 PRIMARY KEY ("ogc_fid")
);
-- Create index "imcra_mesoscale_wkb_geometry_geom_idx" to table: "imcra_mesoscale"
CREATE INDEX "imcra_mesoscale_wkb_geometry_geom_idx" ON "public"."imcra_mesoscale" USING gist ("wkb_geometry");
-- Create enum type "attribute_value_type"
CREATE TYPE "public"."attribute_value_type" AS ENUM ('boolean', 'integer', 'decimal', 'string', 'timestamp');
-- Create "jobs" table
CREATE TABLE "public"."jobs" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "status" "public"."job_status" NOT NULL DEFAULT 'pending',
 "worker" character varying(255) NOT NULL,
 "payload" jsonb NULL,
 "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "updated_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 PRIMARY KEY ("id")
);
-- Create enum type "attribute_category"
CREATE TYPE "public"."attribute_category" AS ENUM ('bushfire_recovery');
-- Create "users" table
CREATE TABLE "public"."users" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "name" character varying(255) NOT NULL,
 "email" character varying(255) NOT NULL,
 "user_role" character varying(255) NOT NULL,
 "password_hash" character varying(255) NOT NULL,
 "password_salt" character varying(255) NOT NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "users_email_key" UNIQUE ("email")
);
-- Create "datasets" table
CREATE TABLE "public"."datasets" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "source_id" uuid NOT NULL,
 "global_id" character varying NOT NULL,
 "name" character varying NOT NULL,
 "short_name" character varying NULL,
 "description" text NULL,
 "url" character varying NULL,
 "citation" character varying NULL,
 "license" character varying NULL,
 "rights_holder" character varying NULL,
 "created_at" timestamptz NOT NULL,
 "updated_at" timestamptz NOT NULL,
 "reuse_pill" "public"."data_reuse_status" NULL,
 "access_pill" "public"."access_rights_status" NULL,
 "publication_year" smallint NULL,
 "content_type" "public"."source_content_type" NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "datasets_source_id_fkey" FOREIGN KEY ("source_id") REFERENCES "public"."sources" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "dataset_global_id" to table: "datasets"
CREATE UNIQUE INDEX "dataset_global_id" ON "public"."datasets" ("global_id");
-- Create "names" table
CREATE TABLE "public"."names" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "scientific_name" character varying NOT NULL,
 "canonical_name" character varying NOT NULL,
 "authorship" character varying NULL,
 PRIMARY KEY ("id")
);
-- Create index "names_canonical_name" to table: "names"
CREATE INDEX "names_canonical_name" ON "public"."names" ("canonical_name");
-- Create index "names_scientific_name" to table: "names"
CREATE UNIQUE INDEX "names_scientific_name" ON "public"."names" ("scientific_name");
-- Set comment to table: "names"
COMMENT ON TABLE "public"."names" IS 'All taxa names. Unique names used to associate attributes and data for specific taxonomic names';
-- Create "specimens" table
CREATE TABLE "public"."specimens" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "name_id" uuid NOT NULL,
 "record_id" character varying NOT NULL,
 "material_sample_id" character varying NULL,
 "organism_id" character varying NULL,
 "institution_name" character varying NULL,
 "institution_code" character varying NULL,
 "collection_code" character varying NULL,
 "recorded_by" character varying NULL,
 "identified_by" character varying NULL,
 "identified_date" character varying NULL,
 "type_status" character varying NULL,
 "locality" character varying NULL,
 "country" character varying NULL,
 "country_code" character varying NULL,
 "state_province" character varying NULL,
 "county" character varying NULL,
 "municipality" character varying NULL,
 "latitude" double precision NULL,
 "longitude" double precision NULL,
 "elevation" double precision NULL,
 "depth" double precision NULL,
 "elevation_accuracy" double precision NULL,
 "depth_accuracy" double precision NULL,
 "location_source" character varying NULL,
 "details" character varying NULL,
 "remarks" character varying NULL,
 "identification_remarks" character varying NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "specimens_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "specimens_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create index "specimens_dataset_id" to table: "specimens"
CREATE INDEX "specimens_dataset_id" ON "public"."specimens" ("dataset_id");
-- Create index "specimens_name_id" to table: "specimens"
CREATE INDEX "specimens_name_id" ON "public"."specimens" ("name_id");
-- Create "accession_events" table
CREATE TABLE "public"."accession_events" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "specimen_id" uuid NOT NULL,
 "event_date" character varying NULL,
 "event_time" character varying NULL,
 "accession" character varying NOT NULL,
 "accessioned_by" character varying NULL,
 "material_sample_id" character varying NULL,
 "institution_name" character varying NULL,
 "institution_code" character varying NULL,
 "type_status" character varying NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "accession_events_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "accession_events_specimen_id_fkey" FOREIGN KEY ("specimen_id") REFERENCES "public"."specimens" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "accession_events_specimen_id" to table: "accession_events"
CREATE INDEX "accession_events_specimen_id" ON "public"."accession_events" ("specimen_id");
-- Create "admin_media" table
CREATE TABLE "public"."admin_media" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "name_id" uuid NOT NULL,
 "image_source" character varying NOT NULL,
 "url" character varying NOT NULL,
 "width" integer NULL,
 "height" integer NULL,
 "reference_url" character varying NULL,
 "title" character varying NULL,
 "description" character varying NULL,
 "source" character varying NULL,
 "creator" character varying NULL,
 "publisher" character varying NULL,
 "license" character varying NULL,
 "rights_holder" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "admin_media_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "subsamples" table
CREATE TABLE "public"."subsamples" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "name_id" uuid NOT NULL,
 "specimen_id" uuid NOT NULL,
 "record_id" character varying NOT NULL,
 "material_sample_id" character varying NULL,
 "institution_name" character varying NULL,
 "institution_code" character varying NULL,
 "type_status" character varying NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "subsamples_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "subsamples_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "subsamples_specimen_id_fkey" FOREIGN KEY ("specimen_id") REFERENCES "public"."specimens" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "subsamples_dataset_id" to table: "subsamples"
CREATE INDEX "subsamples_dataset_id" ON "public"."subsamples" ("dataset_id");
-- Create index "subsamples_name_id" to table: "subsamples"
CREATE INDEX "subsamples_name_id" ON "public"."subsamples" ("name_id");
-- Create index "subsamples_specimen_id" to table: "subsamples"
CREATE INDEX "subsamples_specimen_id" ON "public"."subsamples" ("specimen_id");
-- Create "dna_extracts" table
CREATE TABLE "public"."dna_extracts" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "name_id" uuid NOT NULL,
 "subsample_id" uuid NOT NULL,
 "record_id" character varying NOT NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "dna_extracts_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "dna_extracts_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "dna_extracts_subsample_id_fkey" FOREIGN KEY ("subsample_id") REFERENCES "public"."subsamples" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "dna_extracts_dataset_id" to table: "dna_extracts"
CREATE INDEX "dna_extracts_dataset_id" ON "public"."dna_extracts" ("dataset_id");
-- Create index "dna_extracts_name_id" to table: "dna_extracts"
CREATE INDEX "dna_extracts_name_id" ON "public"."dna_extracts" ("name_id");
-- Create index "dna_extracts_subsample_id" to table: "dna_extracts"
CREATE INDEX "dna_extracts_subsample_id" ON "public"."dna_extracts" ("subsample_id");
-- Create "sequences" table
CREATE TABLE "public"."sequences" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "name_id" uuid NOT NULL,
 "dna_extract_id" uuid NOT NULL,
 "record_id" character varying NOT NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "sequences_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "sequences_dna_extract_id_fkey" FOREIGN KEY ("dna_extract_id") REFERENCES "public"."dna_extracts" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "sequences_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create index "sequences_dataset_id" to table: "sequences"
CREATE INDEX "sequences_dataset_id" ON "public"."sequences" ("dataset_id");
-- Create index "sequences_dna_extract_id" to table: "sequences"
CREATE INDEX "sequences_dna_extract_id" ON "public"."sequences" ("dna_extract_id");
-- Create index "sequences_name_id" to table: "sequences"
CREATE INDEX "sequences_name_id" ON "public"."sequences" ("name_id");
-- Create index "sequences_unique_record_id" to table: "sequences"
CREATE UNIQUE INDEX "sequences_unique_record_id" ON "public"."sequences" ("dataset_id", "record_id");
-- Create "annotation_events" table
CREATE TABLE "public"."annotation_events" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "sequence_id" uuid NOT NULL,
 "event_date" character varying NULL,
 "event_time" character varying NULL,
 "annotated_by" character varying NULL,
 "representation" character varying NULL,
 "release_type" character varying NULL,
 "coverage" character varying NULL,
 "replicons" bigint NULL,
 "standard_operating_procedures" character varying NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "annotation_events_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "annotation_events_sequence_id_fkey" FOREIGN KEY ("sequence_id") REFERENCES "public"."sequences" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "annotation_events_sequence_id" to table: "annotation_events"
CREATE INDEX "annotation_events_sequence_id" ON "public"."annotation_events" ("sequence_id");
-- Create "assemblies" table
CREATE TABLE "public"."assemblies" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "name_id" uuid NOT NULL,
 "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "updated_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "accession" character varying NOT NULL,
 "bioproject_id" character varying NULL,
 "biosample_id" character varying NULL,
 "material_sample_id" character varying NULL,
 "nuccore" character varying NULL,
 "refseq_category" character varying NULL,
 "specific_host" character varying NULL,
 "clone_strain" character varying NULL,
 "version_status" character varying NULL,
 "contam_screen_input" character varying NULL,
 "release_type" character varying NULL,
 "genome_rep" character varying NULL,
 "gbrs_paired_asm" character varying NULL,
 "paired_asm_comp" character varying NULL,
 "excluded_from_refseq" character varying NULL,
 "relation_to_type_material" character varying NULL,
 "asm_not_live_date" character varying NULL,
 "other_catalog_numbers" character varying NULL,
 "recorded_by" character varying NULL,
 "genetic_accession_uri" character varying NULL,
 "event_date" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "assemblies_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "assembly_events" table
CREATE TABLE "public"."assembly_events" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "sequence_id" uuid NOT NULL,
 "event_date" character varying NULL,
 "event_time" character varying NULL,
 "assembled_by" character varying NULL,
 "name" character varying NULL,
 "version_status" character varying NULL,
 "quality" character varying NULL,
 "assembly_type" character varying NULL,
 "genome_size" bigint NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "assembly_events_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "assembly_events_sequence_id_fkey" FOREIGN KEY ("sequence_id") REFERENCES "public"."sequences" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "assembly_events_sequence_id" to table: "assembly_events"
CREATE INDEX "assembly_events_sequence_id" ON "public"."assembly_events" ("sequence_id");
-- Create "assembly_stats" table
CREATE TABLE "public"."assembly_stats" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "assembly_id" uuid NOT NULL,
 "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "updated_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "total_length" integer NULL,
 "spanned_gaps" integer NULL,
 "unspanned_gaps" integer NULL,
 "region_count" integer NULL,
 "scaffold_count" integer NULL,
 "scaffold_n50" integer NULL,
 "scaffold_l50" integer NULL,
 "scaffold_n75" integer NULL,
 "scaffold_n90" integer NULL,
 "contig_count" integer NULL,
 "contig_n50" integer NULL,
 "contig_l50" integer NULL,
 "total_gap_length" integer NULL,
 "molecule_count" integer NULL,
 "top_level_count" integer NULL,
 "component_count" integer NULL,
 "gc_perc" integer NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "assembly_stats_assembly_id_fkey" FOREIGN KEY ("assembly_id") REFERENCES "public"."assemblies" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "biosamples" table
CREATE TABLE "public"."biosamples" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "name_id" uuid NOT NULL,
 "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "updated_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "accession" character varying NOT NULL,
 "sra" character varying NULL,
 "submission_date" character varying NULL,
 "publication_date" character varying NULL,
 "last_update" character varying NULL,
 "title" character varying NULL,
 "owner" character varying NULL,
 "attributes" jsonb NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "biosamples_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "dataset_versions" table
CREATE TABLE "public"."dataset_versions" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "version" character varying NOT NULL,
 "created_at" timestamptz NOT NULL,
 "imported_at" timestamptz NOT NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "dataset_versions_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "collection_event_logs" table
CREATE TABLE "public"."collection_event_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "collection_event_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "collection_event_logs_dataset_version_id" to table: "collection_event_logs"
CREATE INDEX "collection_event_logs_dataset_version_id" ON "public"."collection_event_logs" ("dataset_version_id");
-- Create index "collection_event_logs_entity_id" to table: "collection_event_logs"
CREATE INDEX "collection_event_logs_entity_id" ON "public"."collection_event_logs" ("entity_id");
-- Create index "collection_event_logs_parent_id" to table: "collection_event_logs"
CREATE INDEX "collection_event_logs_parent_id" ON "public"."collection_event_logs" ("parent_id");
-- Create "collection_events" table
CREATE TABLE "public"."collection_events" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "specimen_id" uuid NOT NULL,
 "event_date" character varying NULL,
 "event_time" character varying NULL,
 "collected_by" character varying NULL,
 "field_number" character varying NULL,
 "catalog_number" character varying NULL,
 "record_number" character varying NULL,
 "individual_count" character varying NULL,
 "organism_quantity" character varying NULL,
 "organism_quantity_type" character varying NULL,
 "sex" character varying NULL,
 "genotypic_sex" character varying NULL,
 "phenotypic_sex" character varying NULL,
 "life_stage" character varying NULL,
 "reproductive_condition" character varying NULL,
 "behavior" character varying NULL,
 "establishment_means" character varying NULL,
 "degree_of_establishment" character varying NULL,
 "pathway" character varying NULL,
 "occurrence_status" character varying NULL,
 "preparation" character varying NULL,
 "other_catalog_numbers" character varying NULL,
 "env_broad_scale" character varying NULL,
 "env_local_scale" character varying NULL,
 "env_medium" character varying NULL,
 "habitat" character varying NULL,
 "ref_biomaterial" character varying NULL,
 "source_mat_id" character varying NULL,
 "specific_host" character varying NULL,
 "strain" character varying NULL,
 "isolate" character varying NULL,
 "field_notes" character varying NULL,
 "remarks" character varying NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "collection_events_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "collection_events_specimen_id_fkey" FOREIGN KEY ("specimen_id") REFERENCES "public"."specimens" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "collection_events_specimen_id" to table: "collection_events"
CREATE INDEX "collection_events_specimen_id" ON "public"."collection_events" ("specimen_id");
-- Create "deposition_events" table
CREATE TABLE "public"."deposition_events" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "sequence_id" uuid NOT NULL,
 "event_date" character varying NULL,
 "event_time" character varying NULL,
 "accession" character varying NULL,
 "submitted_by" character varying NULL,
 "material_sample_id" character varying NULL,
 "collection_name" character varying NULL,
 "collection_code" character varying NULL,
 "institution_name" character varying NULL,
 "data_type" character varying NULL,
 "excluded_from_refseq" character varying NULL,
 "asm_not_live_date" character varying NULL,
 "source_uri" character varying NULL,
 "title" character varying NULL,
 "url" character varying NULL,
 "funding_attribution" character varying NULL,
 "rights_holder" character varying NULL,
 "access_rights" character varying NULL,
 "reference" character varying NULL,
 "last_updated" date NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "deposition_events_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "deposition_events_sequence_id_fkey" FOREIGN KEY ("sequence_id") REFERENCES "public"."sequences" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "deposition_events_sequence_id" to table: "deposition_events"
CREATE INDEX "deposition_events_sequence_id" ON "public"."deposition_events" ("sequence_id");
-- Create "dna_extraction_events" table
CREATE TABLE "public"."dna_extraction_events" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "dna_extract_id" uuid NOT NULL,
 "event_date" character varying NULL,
 "event_time" character varying NULL,
 "extracted_by" character varying NULL,
 "preservation_type" character varying NULL,
 "preparation_type" character varying NULL,
 "extraction_method" character varying NULL,
 "measurement_method" character varying NULL,
 "concentration_method" character varying NULL,
 "quality" character varying NULL,
 "concentration" double precision NULL,
 "absorbance_260_230" double precision NULL,
 "absorbance_260_280" double precision NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "dna_extraction_events_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "dna_extraction_events_dna_extract_id_fkey" FOREIGN KEY ("dna_extract_id") REFERENCES "public"."dna_extracts" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "dna_extraction_events_dna_extracts_id" to table: "dna_extraction_events"
CREATE INDEX "dna_extraction_events_dna_extracts_id" ON "public"."dna_extraction_events" ("dna_extract_id");
-- Create "ecology" table
CREATE TABLE "public"."ecology" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "name_id" uuid NOT NULL,
 "values" text[] NOT NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "ecology_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "ecology_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "indigenous_knowledge" table
CREATE TABLE "public"."indigenous_knowledge" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "name_id" uuid NOT NULL,
 "name" character varying NOT NULL,
 "food_use" boolean NOT NULL,
 "medicinal_use" boolean NOT NULL,
 "cultural_connection" boolean NOT NULL,
 "last_updated" timestamptz NOT NULL,
 "source_url" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "indigenous_knowledge_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "indigenous_knowledge_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "name_attributes" table
CREATE TABLE "public"."name_attributes" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "name_id" uuid NOT NULL,
 "name" character varying NOT NULL,
 "category" "public"."attribute_category" NOT NULL,
 "value_type" "public"."attribute_value_type" NOT NULL,
 "value_bool" boolean NULL,
 "value_int" bigint NULL,
 "value_decimal" numeric NULL,
 "value_str" character varying NULL,
 "value_timestamp" timestamp NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "name_attributes_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "name_attributes_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create index "name_attributes_unique_name" to table: "name_attributes"
CREATE UNIQUE INDEX "name_attributes_unique_name" ON "public"."name_attributes" ("dataset_id", "name_id", "name");
-- Create "name_publications" table
CREATE TABLE "public"."name_publications" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "citation" character varying NULL,
 "published_year" integer NULL,
 "source_url" character varying NULL,
 "type_citation" character varying NULL,
 "record_created_at" timestamptz NULL,
 "record_updated_at" timestamptz NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "name_publications_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "nomenclatural_act_logs" table
CREATE TABLE "public"."nomenclatural_act_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "nomenclatural_act_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "nomenclatural_act_logs_dataset_version_id" to table: "nomenclatural_act_logs"
CREATE INDEX "nomenclatural_act_logs_dataset_version_id" ON "public"."nomenclatural_act_logs" ("dataset_version_id");
-- Create index "nomenclatural_act_logs_entity_id" to table: "nomenclatural_act_logs"
CREATE INDEX "nomenclatural_act_logs_entity_id" ON "public"."nomenclatural_act_logs" ("entity_id");
-- Create index "nomenclatural_act_logs_parent_id" to table: "nomenclatural_act_logs"
CREATE INDEX "nomenclatural_act_logs_parent_id" ON "public"."nomenclatural_act_logs" ("parent_id");
-- Create "publications" table
CREATE TABLE "public"."publications" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "entity_id" character varying NOT NULL,
 "title" character varying NOT NULL,
 "authors" text[] NOT NULL,
 "published_year" integer NOT NULL,
 "published_date" timestamptz NULL,
 "language" character varying NULL,
 "publisher" character varying NULL,
 "doi" character varying NULL,
 "source_urls" text[] NULL,
 "publication_type" "public"."publication_type" NULL,
 "citation" character varying NULL,
 "record_created_at" timestamptz NULL,
 "record_updated_at" timestamptz NULL,
 "created_at" timestamptz NOT NULL,
 "updated_at" timestamptz NOT NULL,
 PRIMARY KEY ("id")
);
-- Create index "publications_entity_id" to table: "publications"
CREATE UNIQUE INDEX "publications_entity_id" ON "public"."publications" ("entity_id");
-- Create "nomenclatural_acts" table
CREATE TABLE "public"."nomenclatural_acts" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "entity_id" character varying NOT NULL,
 "name_id" uuid NOT NULL,
 "acted_on_id" uuid NOT NULL,
 "act" "public"."nomenclatural_act_type" NOT NULL,
 "source_url" character varying NOT NULL,
 "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "updated_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "publication_id" uuid NOT NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "nomenclatural_acts_acted_on_id_fkey" FOREIGN KEY ("acted_on_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "nomenclatural_acts_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "nomenclatural_acts_publication_id_fkey" FOREIGN KEY ("publication_id") REFERENCES "public"."publications" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create index "nomenclatrural_acts_name" to table: "nomenclatural_acts"
CREATE INDEX "nomenclatrural_acts_name" ON "public"."nomenclatural_acts" ("name_id");
-- Create index "nomenclatural_acts_acted_on" to table: "nomenclatural_acts"
CREATE INDEX "nomenclatural_acts_acted_on" ON "public"."nomenclatural_acts" ("acted_on_id");
-- Create index "nomenclatural_acts_unique_entity" to table: "nomenclatural_acts"
CREATE UNIQUE INDEX "nomenclatural_acts_unique_entity" ON "public"."nomenclatural_acts" ("entity_id");
-- Set comment to table: "nomenclatural_acts"
COMMENT ON TABLE "public"."nomenclatural_acts" IS 'Name definitions and redefinitions. Any act on a name';
-- Set comment to column: "entity_id" on table: "nomenclatural_acts"
COMMENT ON COLUMN "public"."nomenclatural_acts"."entity_id" IS 'The entity in the logs that this record is reduced from';
-- Set comment to column: "name_id" on table: "nomenclatural_acts"
COMMENT ON COLUMN "public"."nomenclatural_acts"."name_id" IS 'The name that has been defined or changed';
-- Set comment to column: "acted_on_id" on table: "nomenclatural_acts"
COMMENT ON COLUMN "public"."nomenclatural_acts"."acted_on_id" IS 'The name that is being affected by this act';
-- Set comment to column: "act" on table: "nomenclatural_acts"
COMMENT ON COLUMN "public"."nomenclatural_acts"."act" IS 'The specific act being performed by this record';
-- Create "organisms" table
CREATE TABLE "public"."organisms" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "name_id" uuid NOT NULL,
 "organism_id" character varying NULL,
 "organism_name" character varying NULL,
 "organism_scope" character varying NULL,
 "associated_organisms" character varying NULL,
 "previous_identifications" character varying NULL,
 "remarks" text NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "organisms_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "publication_logs" table
CREATE TABLE "public"."publication_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "publication_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "publication_logs_dataset_version_id" to table: "publication_logs"
CREATE INDEX "publication_logs_dataset_version_id" ON "public"."publication_logs" ("dataset_version_id");
-- Create index "publication_logs_entity_id" to table: "publication_logs"
CREATE INDEX "publication_logs_entity_id" ON "public"."publication_logs" ("entity_id");
-- Create index "publication_logs_parent_id" to table: "publication_logs"
CREATE INDEX "publication_logs_parent_id" ON "public"."publication_logs" ("parent_id");
-- Create "regions" table
CREATE TABLE "public"."regions" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "name_id" uuid NOT NULL,
 "region_type" "public"."region_type" NOT NULL,
 "values" text[] NOT NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "regions_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "regions_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "sequence_logs" table
CREATE TABLE "public"."sequence_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "sequence_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "sequence_logs_dataset_version_id" to table: "sequence_logs"
CREATE INDEX "sequence_logs_dataset_version_id" ON "public"."sequence_logs" ("dataset_version_id");
-- Create index "sequence_logs_entity_id" to table: "sequence_logs"
CREATE INDEX "sequence_logs_entity_id" ON "public"."sequence_logs" ("entity_id");
-- Create index "sequence_logs_parent_id" to table: "sequence_logs"
CREATE INDEX "sequence_logs_parent_id" ON "public"."sequence_logs" ("parent_id");
-- Create "sequencing_events" table
CREATE TABLE "public"."sequencing_events" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "sequence_id" uuid NOT NULL,
 "event_date" character varying NULL,
 "event_time" character varying NULL,
 "sequenced_by" character varying NULL,
 "material_sample_id" character varying NULL,
 "concentration" double precision NULL,
 "amplicon_size" bigint NULL,
 "estimated_size" character varying NULL,
 "bait_set_name" character varying NULL,
 "bait_set_reference" character varying NULL,
 "target_gene" character varying NULL,
 "dna_sequence" text NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "sequencing_events_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "sequencing_events_sequence_id_fkey" FOREIGN KEY ("sequence_id") REFERENCES "public"."sequences" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "sequencing_events_sequence_id" to table: "sequencing_events"
CREATE INDEX "sequencing_events_sequence_id" ON "public"."sequencing_events" ("sequence_id");
-- Create "sequencing_run_events" table
CREATE TABLE "public"."sequencing_run_events" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "sequencing_event_id" uuid NOT NULL,
 "trace_id" character varying NULL,
 "trace_name" character varying NULL,
 "trace_link" character varying NULL,
 "sequencing_date" timestamp NULL,
 "sequencing_center" character varying NULL,
 "sequencing_center_code" character varying NULL,
 "sequencing_method" character varying NULL,
 "target_gene" character varying NULL,
 "direction" character varying NULL,
 "pcr_primer_name_forward" character varying NULL,
 "pcr_primer_name_reverse" character varying NULL,
 "sequence_primer_forward_name" character varying NULL,
 "sequence_primer_reverse_name" character varying NULL,
 "library_protocol" character varying NULL,
 "analysis_description" character varying NULL,
 "analysis_software" character varying NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "sequencing_run_events_sequencing_event_id_fkey" FOREIGN KEY ("sequencing_event_id") REFERENCES "public"."sequencing_events" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "sequencing_run_ev_seq_event_id" to table: "sequencing_run_events"
CREATE INDEX "sequencing_run_ev_seq_event_id" ON "public"."sequencing_run_events" ("sequencing_event_id");
-- Create "specimen_logs" table
CREATE TABLE "public"."specimen_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "specimen_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "specimen_logs_dataset_version_id" to table: "specimen_logs"
CREATE INDEX "specimen_logs_dataset_version_id" ON "public"."specimen_logs" ("dataset_version_id");
-- Create index "specimen_logs_entity_id" to table: "specimen_logs"
CREATE INDEX "specimen_logs_entity_id" ON "public"."specimen_logs" ("entity_id");
-- Create index "specimen_logs_parent_id" to table: "specimen_logs"
CREATE INDEX "specimen_logs_parent_id" ON "public"."specimen_logs" ("parent_id");
-- Create "subsample_events" table
CREATE TABLE "public"."subsample_events" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "subsample_id" uuid NOT NULL,
 "event_date" character varying NULL,
 "event_time" character varying NULL,
 "subsampled_by" character varying NULL,
 "preparation_type" character varying NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "subsample_events_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "subsample_events_subsample_id_fkey" FOREIGN KEY ("subsample_id") REFERENCES "public"."subsamples" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "subsample_events_subsample_id" to table: "subsample_events"
CREATE INDEX "subsample_events_subsample_id" ON "public"."subsample_events" ("subsample_id");
-- Create "taxa" table
CREATE TABLE "public"."taxa" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "parent_id" uuid NULL,
 "status" "public"."taxonomic_status" NOT NULL,
 "rank" "public"."taxonomic_rank" NOT NULL,
 "scientific_name" character varying NOT NULL,
 "canonical_name" character varying NOT NULL,
 "authorship" character varying NULL,
 "nomenclatural_code" character varying NOT NULL,
 "citation" character varying NULL,
 "vernacular_names" text[] NULL,
 "description" text NULL,
 "remarks" text NULL,
 "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "updated_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "taxa_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "taxa_parent_id_fkey" FOREIGN KEY ("parent_id") REFERENCES "public"."taxa" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create index "taxa_parent_id" to table: "taxa"
CREATE INDEX "taxa_parent_id" ON "public"."taxa" ("parent_id");
-- Create index "taxa_unique_name" to table: "taxa"
CREATE UNIQUE INDEX "taxa_unique_name" ON "public"."taxa" ("scientific_name", "dataset_id");
-- Create "taxa_logs" table
CREATE TABLE "public"."taxa_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "taxa_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "taxa_logs_dataset_version_id" to table: "taxa_logs"
CREATE INDEX "taxa_logs_dataset_version_id" ON "public"."taxa_logs" ("dataset_version_id");
-- Create index "taxa_logs_entity_id" to table: "taxa_logs"
CREATE INDEX "taxa_logs_entity_id" ON "public"."taxa_logs" ("entity_id");
-- Create index "taxa_logs_parent_id" to table: "taxa_logs"
CREATE INDEX "taxa_logs_parent_id" ON "public"."taxa_logs" ("parent_id");
-- Create "taxon_history" table
CREATE TABLE "public"."taxon_history" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "acted_on" uuid NOT NULL,
 "taxon_id" uuid NOT NULL,
 "dataset_id" uuid NOT NULL,
 "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "updated_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "publication_id" uuid NULL,
 "source_url" character varying NULL,
 "entity_id" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "taxon_history_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "taxon_history_new_taxon_id_fkey" FOREIGN KEY ("taxon_id") REFERENCES "public"."taxa" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "taxon_history_old_taxon_id_fkey" FOREIGN KEY ("acted_on") REFERENCES "public"."taxa" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "taxon_history_publication_id_fkey" FOREIGN KEY ("publication_id") REFERENCES "public"."name_publications" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create index "taxon_history_unique_link" to table: "taxon_history"
CREATE UNIQUE INDEX "taxon_history_unique_link" ON "public"."taxon_history" ("acted_on", "taxon_id");
-- Create "taxon_names" table
CREATE TABLE "public"."taxon_names" (
 "taxon_id" uuid NOT NULL,
 "name_id" uuid NOT NULL,
 PRIMARY KEY ("taxon_id", "name_id"),
 CONSTRAINT "taxon_names_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "taxon_names_taxon_id_fkey" FOREIGN KEY ("taxon_id") REFERENCES "public"."taxa" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "taxon_photos" table
CREATE TABLE "public"."taxon_photos" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "taxon_id" uuid NOT NULL,
 "url" character varying NOT NULL,
 "source" character varying NULL,
 "publisher" character varying NULL,
 "license" character varying NULL,
 "rights_holder" character varying NULL,
 "priority" integer NOT NULL DEFAULT 1,
 PRIMARY KEY ("id"),
 CONSTRAINT "taxon_photos_taxon_id_fkey" FOREIGN KEY ("taxon_id") REFERENCES "public"."taxa" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "taxonomic_act_logs" table
CREATE TABLE "public"."taxonomic_act_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "taxonomic_act_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "taxonomic_act_logs_dataset_version_id" to table: "taxonomic_act_logs"
CREATE INDEX "taxonomic_act_logs_dataset_version_id" ON "public"."taxonomic_act_logs" ("dataset_version_id");
-- Create index "taxonomic_act_logs_entity_id" to table: "taxonomic_act_logs"
CREATE INDEX "taxonomic_act_logs_entity_id" ON "public"."taxonomic_act_logs" ("entity_id");
-- Create index "taxonomic_act_logs_parent_id" to table: "taxonomic_act_logs"
CREATE INDEX "taxonomic_act_logs_parent_id" ON "public"."taxonomic_act_logs" ("parent_id");
-- Create "taxonomic_acts" table
CREATE TABLE "public"."taxonomic_acts" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "entity_id" character varying NOT NULL,
 "taxon_id" uuid NOT NULL,
 "accepted_taxon_id" uuid NULL,
 "source_url" character varying NULL,
 "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "updated_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "data_created_at" timestamptz NULL,
 "data_updated_at" timestamptz NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "taxonomic_acts_accepted_taxon_id_fkey" FOREIGN KEY ("accepted_taxon_id") REFERENCES "public"."taxa" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "taxonomic_acts_taxon_id_fkey" FOREIGN KEY ("taxon_id") REFERENCES "public"."taxa" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create index "taxonomic_acts_accepted_taxon" to table: "taxonomic_acts"
CREATE INDEX "taxonomic_acts_accepted_taxon" ON "public"."taxonomic_acts" ("accepted_taxon_id");
-- Create index "taxonomic_acts_unique_entity" to table: "taxonomic_acts"
CREATE UNIQUE INDEX "taxonomic_acts_unique_entity" ON "public"."taxonomic_acts" ("entity_id");
-- Create index "taxonomic_acts_unique_taxon" to table: "taxonomic_acts"
CREATE INDEX "taxonomic_acts_unique_taxon" ON "public"."taxonomic_acts" ("taxon_id");
-- Set comment to table: "taxonomic_acts"
COMMENT ON TABLE "public"."taxonomic_acts" IS 'An act within a specific taxonomic system';
-- Set comment to column: "entity_id" on table: "taxonomic_acts"
COMMENT ON COLUMN "public"."taxonomic_acts"."entity_id" IS 'The entity in the logs that this record is reduced from';
-- Set comment to column: "taxon_id" on table: "taxonomic_acts"
COMMENT ON COLUMN "public"."taxonomic_acts"."taxon_id" IS 'The taxon that is being affected by this act';
-- Set comment to column: "accepted_taxon_id" on table: "taxonomic_acts"
COMMENT ON COLUMN "public"."taxonomic_acts"."accepted_taxon_id" IS 'The taxon that is considered currently accepted in the system';
-- Create "trace_files" table
CREATE TABLE "public"."trace_files" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "name_id" uuid NOT NULL,
 "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "updated_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
 "metadata" jsonb NOT NULL,
 "peak_locations_user" integer[] NULL,
 "peak_locations_basecaller" integer[] NULL,
 "quality_values_user" integer[] NULL,
 "quality_values_basecaller" integer[] NULL,
 "sequences_user" integer[] NULL,
 "sequences_basecaller" integer[] NULL,
 "measurements_voltage" integer[] NULL,
 "measurements_current" integer[] NULL,
 "measurements_power" integer[] NULL,
 "measurements_temperature" integer[] NULL,
 "analyzed_g" integer[] NULL,
 "analyzed_a" integer[] NULL,
 "analyzed_t" integer[] NULL,
 "analyzed_c" integer[] NULL,
 "raw_g" integer[] NULL,
 "raw_a" integer[] NULL,
 "raw_t" integer[] NULL,
 "raw_c" integer[] NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "trace_files_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "trace_files_analyzed_a_check" CHECK (array_position(analyzed_a, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_analyzed_c_check" CHECK (array_position(analyzed_c, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_analyzed_g_check" CHECK (array_position(analyzed_g, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_analyzed_t_check" CHECK (array_position(analyzed_t, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_measurements_current_check" CHECK (array_position(measurements_current, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_measurements_power_check" CHECK (array_position(measurements_power, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_measurements_temperature_check" CHECK (array_position(measurements_temperature, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_measurements_voltage_check" CHECK (array_position(measurements_voltage, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_peak_locations_basecaller_check" CHECK (array_position(peak_locations_basecaller, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_peak_locations_user_check" CHECK (array_position(peak_locations_user, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_quality_values_basecaller_check" CHECK (array_position(quality_values_basecaller, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_quality_values_user_check" CHECK (array_position(quality_values_user, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_raw_a_check" CHECK (array_position(raw_a, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_raw_c_check" CHECK (array_position(raw_c, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_raw_g_check" CHECK (array_position(raw_g, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_raw_t_check" CHECK (array_position(raw_t, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_sequences_basecaller_check" CHECK (array_position(sequences_basecaller, NULL::integer) IS NULL),
 CONSTRAINT "trace_files_sequences_user_check" CHECK (array_position(sequences_user, NULL::integer) IS NULL)
);
-- Create "vernacular_names" table
CREATE TABLE "public"."vernacular_names" (
 "id" uuid NOT NULL DEFAULT gen_random_uuid(),
 "dataset_id" uuid NOT NULL,
 "name_id" uuid NOT NULL,
 "vernacular_name" character varying NOT NULL,
 "citation" character varying NULL,
 "source_url" character varying NULL,
 PRIMARY KEY ("id"),
 CONSTRAINT "vernacular_names_dataset_id_fkey" FOREIGN KEY ("dataset_id") REFERENCES "public"."datasets" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "vernacular_names_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create index "vernacular_names_unique_name" to table: "vernacular_names"
CREATE UNIQUE INDEX "vernacular_names_unique_name" ON "public"."vernacular_names" ("dataset_id", "name_id", "vernacular_name");
