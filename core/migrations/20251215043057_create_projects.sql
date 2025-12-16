-- Create enum type "project_member_role_type"
CREATE TYPE "public"."project_member_role_type" AS ENUM ('lead');
-- Create "projects" table
CREATE TABLE "public"."projects" (
 "entity_id" character varying NOT NULL,
 "project_id" character varying NULL,
 "target_species_name_id" bigint NULL,
 "title" character varying NULL,
 "description" text NULL,
 "initiative" character varying NULL,
 "registration_date" date NULL,
 "data_context" character varying[] NULL,
 "data_types" character varying[] NULL,
 "data_assay_types" character varying[] NULL,
 "partners" character varying[] NULL,
 PRIMARY KEY ("entity_id")
);
-- Create index "projects_target_species_name_id" to table: "projects"
CREATE INDEX "projects_target_species_name_id" ON "public"."projects" ("target_species_name_id");
-- Create "project_members" table
CREATE TABLE "public"."project_members" (
 "project_entity_id" character varying NOT NULL,
 "agent_entity_id" character varying NOT NULL,
 "organisation" character varying NULL,
 "project_role" "public"."project_member_role_type" NOT NULL,
 PRIMARY KEY ("project_entity_id", "agent_entity_id"),
 CONSTRAINT "project_members_agent_entity_id_fkey" FOREIGN KEY ("agent_entity_id") REFERENCES "public"."agents" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "project_members_project_entity_id_fkey" FOREIGN KEY ("project_entity_id") REFERENCES "public"."projects" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "project_members_agent_entity_id" to table: "project_members"
CREATE INDEX "project_members_agent_entity_id" ON "public"."project_members" ("agent_entity_id");
-- Create index "project_members_project_entity_id" to table: "project_members"
CREATE INDEX "project_members_project_entity_id" ON "public"."project_members" ("project_entity_id");
