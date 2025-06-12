---------------------------
-- Views
---------------------------

-- a convenience view that joins all the sequencing events. Will return duplicates of
-- a sequence if there are multiple events of the same type for a sequence
CREATE OR REPLACE VIEW whole_genomes AS
SELECT
    sequences.id AS sequence_id,
    sequences.dataset_id,
    sequences.name_id,
    sequences.dna_extract_id,
    datasets.name AS dataset_name,
    sequences.record_id,
    specimens.latitude,
    specimens.longitude,
    deposition_events.accession,
    sequencing_events.sequenced_by,
    sequencing_events.material_sample_id,
    sequencing_events.estimated_size,
    assembly_events.assembled_by,
    assembly_events.name,
    assembly_events.version_status,
    assembly_events.quality,
    assembly_events.assembly_type,
    assembly_events.genome_size,
    annotation_events.annotated_by,
    annotation_events.representation,
    annotation_events.release_type,
    deposition_events.event_date AS release_date,
    deposition_events.submitted_by AS deposited_by,
    deposition_events.data_type,
    deposition_events.excluded_from_refseq
FROM sequences
JOIN datasets ON sequences.dataset_id = datasets.id
JOIN sequencing_events ON sequences.id = sequencing_events.sequence_id
JOIN assembly_events ON sequences.id = assembly_events.sequence_id
JOIN annotation_events ON sequences.id = annotation_events.sequence_id
JOIN deposition_events ON sequences.id = deposition_events.sequence_id
LEFT JOIN dna_extracts ON sequences.dna_extract_id = dna_extracts.id
LEFT JOIN subsamples ON dna_extracts.subsample_id = subsamples.id
LEFT JOIN specimens ON subsamples.specimen_id = specimens.id;


-- All loci data
CREATE OR REPLACE VIEW markers AS
SELECT DISTINCT
    sequences.id AS sequence_id,
    sequences.dataset_id,
    sequences.name_id,
    sequences.dna_extract_id,
    datasets.name AS dataset_name,
    sequences.record_id,
    specimens.latitude,
    specimens.longitude,
    deposition_events.accession,
    sequencing_events.sequenced_by,
    sequencing_events.material_sample_id,
    sequencing_events.target_gene,
    deposition_events.event_date AS release_date
FROM sequences
JOIN datasets ON sequences.dataset_id = datasets.id
JOIN sequencing_events ON sequences.id = sequencing_events.sequence_id
LEFT JOIN deposition_events ON sequences.id = deposition_events.sequence_id
LEFT JOIN dna_extracts ON sequences.dna_extract_id = dna_extracts.id
LEFT JOIN subsamples ON dna_extracts.subsample_id = subsamples.id
LEFT JOIN specimens ON subsamples.specimen_id = specimens.id
WHERE sequencing_events.target_gene IS NOT NULL;


-- All data regarded as a genomic component
CREATE OR REPLACE VIEW genomic_components AS
SELECT DISTINCT
    sequences.id AS sequence_id,
    sequences.dataset_id,
    sequences.name_id,
    sequences.dna_extract_id,
    datasets.name AS dataset_name,
    sequences.record_id,
    specimens.latitude,
    specimens.longitude,
    deposition_events.accession,
    sequencing_events.sequenced_by,
    sequencing_events.material_sample_id,
    sequencing_events.estimated_size,
    deposition_events.event_date AS release_date,
    deposition_events.submitted_by AS deposited_by,
    deposition_events.data_type,
    deposition_events.title,
    deposition_events.url,
    deposition_events.source_uri,
    deposition_events.funding_attribution,
    deposition_events.rights_holder,
    deposition_events.access_rights
FROM sequences
JOIN datasets ON sequences.dataset_id = datasets.id
JOIN sequencing_events ON sequences.id = sequencing_events.sequence_id
JOIN deposition_events ON sequences.id = deposition_events.sequence_id
LEFT JOIN assembly_events on sequences.id = assembly_events.sequence_id
LEFT JOIN dna_extracts ON sequences.dna_extract_id = dna_extracts.id
LEFT JOIN subsamples ON dna_extracts.subsample_id = subsamples.id
LEFT JOIN specimens ON subsamples.specimen_id = specimens.id
WHERE assembly_events.id IS NULL AND target_gene IS NULL;


---------------------------
-- Materialized views
---------------------------

-- The earliest dates for specific events within a sequence
CREATE MATERIALIZED VIEW IF NOT EXISTS sequence_milestones AS
SELECT
    sequences.name_id,
    annotation_events.representation,
    MIN(sequencing_events.event_date) AS sequencing_date,
    MIN(assembly_events.event_date) AS assembly_date,
    MIN(annotation_events.event_date) AS annotation_date,
    MIN(deposition_events.event_date) AS deposition_date
FROM sequences
JOIN sequencing_events ON sequences.id = sequencing_events.sequence_id
JOIN assembly_events ON sequences.id = assembly_events.sequence_id
JOIN annotation_events ON sequences.id = annotation_events.sequence_id
JOIN deposition_events ON sequences.id = deposition_events.sequence_id
JOIN taxon_names ON sequences.name_id = taxon_names.name_id
GROUP BY sequences.name_id, representation;

CREATE UNIQUE INDEX IF NOT EXISTS sequence_milestones_name_representation ON sequence_milestones (name_id, representation);


-- Stats for data linked to a name
CREATE MATERIALIZED VIEW IF NOT EXISTS name_data_summaries AS
SELECT
    names.id AS name_id,
    COALESCE(markers.total, 0) AS markers,
    COALESCE(genomes.total, 0) AS genomes,
    COALESCE(specimens.total, 0) AS specimens,
    COALESCE(other_data.total, 0) AS other,
    COALESCE(markers.total, 0) + COALESCE(genomes.total, 0) + COALESCE(other_data.total, 0) AS total_genomic,

    COALESCE(full_genomes.total, 0) AS full_genomes,
    COALESCE(partial_genomes.total, 0) AS partial_genomes,
    COALESCE(complete_genomes.total, 0) AS complete_genomes,
    COALESCE(assembly_chromosomes.total, 0) AS assembly_chromosomes,
    COALESCE(assembly_scaffolds.total, 0) AS assembly_scaffolds,
    COALESCE(assembly_contigs.total, 0) AS assembly_contigs
FROM names
LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM sequencing_events
     JOIN sequences ON sequencing_events.sequence_id = sequences.id
     WHERE target_gene IS NOT NULL
     GROUP BY name_id
) markers ON markers.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM assembly_events
     JOIN sequences ON assembly_events.sequence_id = sequences.id
     GROUP BY name_id
) genomes ON genomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM specimens
     GROUP BY name_id
) specimens ON specimens.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int as total
     FROM sequences
     LEFT JOIN sequencing_events se on sequences.id = se.sequence_id
     LEFT JOIN assembly_events on sequences.id = assembly_events.sequence_id
     LEFT JOIN annotation_events ae on sequences.id = ae.sequence_id
     WHERE assembly_events.id IS NULL AND target_gene IS NULL
     GROUP BY name_id
) other_data ON other_data.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM annotation_events
     JOIN sequences ON annotation_events.sequence_id = sequences.id
     WHERE representation = 'Full'
     GROUP BY name_id
) full_genomes ON full_genomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM annotation_events
     JOIN sequences ON annotation_events.sequence_id = sequences.id
     WHERE representation = 'Partial'
     GROUP BY name_id
) partial_genomes ON partial_genomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM assembly_events
     JOIN sequences ON assembly_events.sequence_id = sequences.id
     WHERE quality = 'Complete Genome'
     GROUP BY name_id
) complete_genomes ON complete_genomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM assembly_events
     JOIN sequences ON assembly_events.sequence_id = sequences.id
     WHERE quality = 'Chromosome'
     GROUP BY name_id
) assembly_chromosomes ON assembly_chromosomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM assembly_events
     JOIN sequences ON assembly_events.sequence_id = sequences.id
     WHERE quality = 'Scaffold'
     GROUP BY name_id
) assembly_scaffolds ON assembly_scaffolds.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM assembly_events
     JOIN sequences ON assembly_events.sequence_id = sequences.id
     WHERE quality = 'Contig'
     GROUP BY name_id
) assembly_contigs ON assembly_contigs.name_id = names.id;

CREATE UNIQUE INDEX IF NOT EXISTS name_data_summaries_name_id ON name_data_summaries (name_id);


-- The primary taxonomy graph. This is a denormalised version of the taxa table
-- which allows us to retrieve the hierarchy of a specific node
CREATE MATERIALIZED VIEW IF NOT EXISTS taxa_dag AS
WITH RECURSIVE dag(
    taxon_id,
    taxon_scientific_name,
    taxon_canonical_name,
    id,
    parent_id,
    rank,
    scientific_name,
    canonical_name,
    depth,
    is_cycle,
    path
) AS (
    SELECT
        id AS taxon_id,
        scientific_name AS taxon_scientific_name,
        canonical_name AS taxon_canonical_name,
        id,
        parent_id,
        rank,
        scientific_name,
        canonical_name,
        0,
        false,
        ARRAY[id]
    FROM taxa
UNION
    SELECT
        dag.taxon_id,
        dag.taxon_scientific_name,
        dag.taxon_canonical_name,
        t.id,
        t.parent_id,
        t.rank,
        t.scientific_name,
        t.canonical_name,
        dag.depth + 1,
        t.id = ANY(path),
        path || t.id
    FROM dag, taxa t
    WHERE dag.parent_id = t.id
      AND dag.id != dag.parent_id
      AND dag.parent_id IS NOT NULL
      AND NOT is_cycle
)
SELECT taxon_id, taxon_scientific_name, taxon_canonical_name, id, parent_id, rank, scientific_name, canonical_name, depth
FROM dag
ORDER BY taxon_id ASC, depth ASC;

CREATE INDEX IF NOT EXISTS taxa_dag_id ON taxa_dag (taxon_id);
CREATE UNIQUE INDEX IF NOT EXISTS taxa_dag_id_taxon_id ON taxa_dag (id, taxon_id);


-- A taxonomy graph derived from taxa_dag to allow querying for all descendants of a specific
-- taxon node
CREATE MATERIALIZED VIEW IF NOT EXISTS taxa_dag_down AS
WITH RECURSIVE dag(
    taxon_id,
    id,
    parent_id,
    depth,
    is_cycle,
    path
) AS (
    SELECT
        id AS taxon_id,
        id,
        parent_id,
        0,         -- depth
        false,     -- is_cycle
        ARRAY[id]  -- path
    FROM taxa
UNION
    -- for each row that the intermediate table spits out we join
    -- on all taxa that has a parent_id matching the rows that were output.
    -- this lets us go down the tree as each time we will output more rows
    -- until we reach the leafs (the row has no other rows linking to it via parent_id).
    SELECT
        dag.taxon_id,
        t.id,
        t.parent_id,
        dag.depth + 1,     -- depth
        t.id = ANY(path),  -- is_cycle
        path || t.id       -- path
    FROM dag, taxa t
    WHERE dag.id = t.parent_id
      -- because we are traversing down the tree we don't need to check for a terminus such as a parent
      -- null check. instead we just want to make sure we aren't infinitely trying to traverse the root
      -- so we only do a cyclic check.
      AND NOT is_cycle
)
SELECT taxon_id, id, parent_id, depth
FROM dag
ORDER BY taxon_id ASC, depth ASC;

COMMENT ON MATERIALIZED VIEW taxa_dag_down IS 'A denormalised graph of all descendents for every taxon';

-- index on the 'query' column. this is how most queries are going to hit the view. specifically to get a list of
-- descendant nodes for a particular taxon
CREATE INDEX IF NOT EXISTS taxa_dag_down_taxon_id ON taxa_dag_down (taxon_id);

-- because the underlying taxa tree is a DAG we know that a taxon can only ever appear once for each taxon_id 'query'.
-- by creating a uniqueness constraint on taxon_id and id we can concurrently update the tree without locking the table
CREATE UNIQUE INDEX IF NOT EXISTS taxa_dag_down_taxon_id_id ON taxa_dag_down (taxon_id, id, depth);


-- A full, denormalised, taxonomic tree
CREATE MATERIALIZED VIEW IF NOT EXISTS taxa_tree AS
WITH RECURSIVE tree (
    taxon_id,
    path_id,
    id,
    parent_id,
    depth,
    path
) AS (
    SELECT
        taxon_id,
        id AS path_id,
        id,
        parent_id,
        0,
        ARRAY[id]
    FROM taxa_dag_down
UNION
    SELECT
        tree.taxon_id,
        tree.path_id,
        t.id,
        t.parent_id,
        tree.depth + 1,
        path || t.id
    FROM tree, taxa t
    WHERE tree.parent_id = t.id
    AND NOT tree.taxon_id = ANY(path)
)
SELECT taxon_id, path_id, id, parent_id, depth
FROM tree
ORDER BY path_id, depth DESC;

COMMENT ON MATERIALIZED VIEW taxa_tree IS 'A denormalised, exhaustive tree containing all paths that descend from every taxon';
COMMENT ON COLUMN taxa_tree.taxon_id IS 'The root taxon that a descending tree is available for';
COMMENT ON COLUMN taxa_tree.path_id IS 'The taxon that this particular path starts from';

CREATE INDEX IF NOT EXISTS taxa_tree_taxon_id ON taxa_tree (taxon_id);
CREATE INDEX IF NOT EXISTS taxa_tree_path_id ON taxa_tree (path_id);
CREATE UNIQUE INDEX IF NOT EXISTS taxa_tree_taxon_id_path_id ON taxa_tree (taxon_id, path_id, depth);


-- Accumulated stats for every node in a taxonomic tree. This will aggregate statistics going
-- so that each node stat is an aggregation of all descendant node stats
CREATE MATERIALIZED VIEW IF NOT EXISTS taxa_tree_stats AS
-- combines the name_data_summaries of names that are linked to one another.
-- this allows us to present data from dark taxa
WITH taxa_data_summaries AS (
     SELECT
        taxon_id,
        SUM(markers) AS loci,
        SUM(genomes) AS genomes,
        SUM(specimens) AS specimens,
        SUM(other) AS other,
        SUM(total_genomic) AS total_genomic,

        SUM(full_genomes) AS full_genomes,
        SUM(partial_genomes) AS partial_genomes,
        SUM(complete_genomes) AS complete_genomes,
        SUM(assembly_chromosomes) AS assembly_chromosomes,
        SUM(assembly_scaffolds) AS assembly_scaffolds,
        SUM(assembly_contigs) AS assembly_contigs
    FROM name_data_summaries
    JOIN taxon_names ON taxon_names.name_id = name_data_summaries.name_id
    GROUP BY taxon_id
),
-- the linked name_data_summaries joined with the taxa_tree to get stats that
-- aggregate up the taxa hierarchy for each taxon
taxon_stats AS (
    SELECT
        taxa_tree.taxon_id,
        id,
        path_id,
        depth,
        -- if the node is the second taxon in the path then its the direct parent of
        -- leaf node, so we use the value of 1 to allow summing when grouped. this allows
        -- us to determine how many direct children each node has
        CASE WHEN depth = 1 THEN 1 ELSE 0 END AS direct_parent,
        -- pull the the values from the name data summaries
        FIRST_VALUE(loci) OVER tree_paths AS loci,
        FIRST_VALUE(genomes) OVER tree_paths AS genomes,
        FIRST_VALUE(specimens) OVER tree_paths AS specimens,
        FIRST_VALUE(other) OVER tree_paths AS other,
        FIRST_VALUE(total_genomic) OVER tree_paths AS total_genomic,
        FIRST_VALUE(full_genomes) OVER tree_paths AS full_genomes,
        FIRST_VALUE(partial_genomes) OVER tree_paths AS partial_genomes,
        FIRST_VALUE(complete_genomes) OVER tree_paths AS complete_genomes,
        FIRST_VALUE(assembly_chromosomes) OVER tree_paths AS assembly_chromosomes,
        FIRST_VALUE(assembly_scaffolds) OVER tree_paths AS assembly_scaffolds,
        FIRST_VALUE(assembly_contigs) OVER tree_paths AS assembly_contigs,
        -- base values for coverage stats. if there is at least one type of data we consider
        -- it full coverage for the node. this is useful further on when summarising a node
        -- and comparing the total coverage against the amount of children to determine coverage
        -- for a node at any part of the hierarchy without losing that information to aggregation
        CASE WHEN FIRST_VALUE(full_genomes) OVER tree_paths > 0 THEN 1 ELSE 0 END AS full_genomes_coverage,
        CASE WHEN FIRST_VALUE(partial_genomes) OVER tree_paths > 0 THEN 1 ELSE 0 END AS partial_genomes_coverage,
        CASE WHEN FIRST_VALUE(complete_genomes) OVER tree_paths > 0 THEN 1 ELSE 0 END AS complete_genomes_coverage,
        CASE WHEN FIRST_VALUE(assembly_chromosomes) OVER tree_paths > 0 THEN 1 ELSE 0 END AS assembly_chromosomes_coverage,
        CASE WHEN FIRST_VALUE(assembly_scaffolds) OVER tree_paths > 0 THEN 1 ELSE 0 END AS assembly_scaffolds_coverage,
        CASE WHEN FIRST_VALUE(assembly_contigs) OVER tree_paths > 0 THEN 1 ELSE 0 END AS assembly_contigs_coverage
    FROM taxa_tree
    -- a taxon can have multiple alternate names so we group them
    -- up and sum it here otherwise it will cause double counting
    LEFT JOIN taxa_data_summaries ON taxa_data_summaries.taxon_id = taxa_tree.id
    WINDOW tree_paths AS (partition BY path_id ORDER BY depth)
    ORDER BY path_id, depth
),
-- the grouped up stats for higher taxonomy. it joins the taxon_stats on the
-- path_id to get the accumulated amounts of all descendents and then groups
-- by the taxon id itself to ensure that all paths are folded in to the parent taxon
stats AS (
    SELECT
        taxon_id,
        taxon_stats.id,
        MAX(depth) AS tree_depth,
        SUM(direct_parent) AS children,
        COUNT(*) - 1 AS descendants,
        SUM(loci) AS loci,
        SUM(genomes) AS genomes,
        SUM(specimens) AS specimens,
        SUM(other) AS other,
        SUM(total_genomic) AS total_genomic,
        SUM(CASE WHEN taxa.rank='species' THEN 1 ELSE 0 END) AS species,
        SUM(full_genomes) AS full_genomes,
        SUM(partial_genomes) AS partial_genomes,
        SUM(complete_genomes) AS complete_genomes,
        SUM(assembly_chromosomes) AS assembly_chromosomes,
        SUM(assembly_scaffolds) AS assembly_scaffolds,
        SUM(assembly_contigs) AS assembly_contigs,
        -- sum up all the coverage for the node and divide it by the amount of children to determine
        -- the total coverage for this specific node.
        --
        -- we want to also clamp the coverage if we are a species node so that the coverage effectively
        -- has a resolution of species and not subspecies or varieties. this is largely a practical matter
        -- as the stability of the tree is much greater at the species level compared to the ranks beneath it
        LEAST(SUM(full_genomes_coverage), SUM(CASE WHEN taxa.rank='species' THEN 1 ELSE 0 END)) AS total_full_genomes_coverage,
        LEAST(SUM(partial_genomes_coverage), SUM(CASE WHEN taxa.rank='species' THEN 1 ELSE 0 END)) AS total_partial_genomes_coverage,
        LEAST(SUM(complete_genomes_coverage), SUM(CASE WHEN taxa.rank='species' THEN 1 ELSE 0 END)) AS total_complete_genomes_coverage,
        LEAST(SUM(assembly_chromosomes_coverage), SUM(CASE WHEN taxa.rank='species' THEN 1 ELSE 0 END)) AS total_assembly_chromosomes_coverage,
        LEAST(SUM(assembly_scaffolds_coverage), SUM(CASE WHEN taxa.rank='species' THEN 1 ELSE 0 END)) AS total_assembly_scaffolds_coverage,
        LEAST(SUM(assembly_contigs_coverage), SUM(CASE WHEN taxa.rank='species' THEN 1 ELSE 0 END)) AS total_assembly_contigs_coverage
    FROM taxon_stats
    JOIN taxa ON taxon_stats.path_id = taxa.id
    GROUP BY taxon_id, taxon_stats.id
)
-- the main query. simply join the tree stats with the taxon and sum the stats for each node
SELECT * FROM stats;

CREATE INDEX IF NOT EXISTS taxa_tree_stats_taxon_id ON taxa_tree_stats (taxon_id);
CREATE INDEX IF NOT EXISTS taxa_tree_stats_id ON taxa_tree_stats (id);
CREATE UNIQUE INDEX IF NOT EXISTS taxa_tree_stats_id_taxon_id ON taxa_tree_stats (id, taxon_id);


-- All species with associated data
CREATE MATERIALIZED VIEW IF NOT EXISTS species AS
SELECT
    taxa.id,
    taxa.scientific_name,
    taxa.canonical_name,
    taxa.authorship,
    taxa.dataset_id,
    taxa.status,
    taxa.rank,
    taxa_tree.classification,
    summaries.genomes,
    summaries.loci,
    summaries.specimens,
    summaries.other,
    summaries.total_genomic,
    name_attributes.traits,
    name_attributes.attributes,
    vernacular_names.names AS vernacular_names
FROM taxa
JOIN (
  SELECT
      taxon_id,
      jsonb_object_agg(rank, canonical_name) AS classification
  FROM taxa_dag
  GROUP BY taxon_id
) taxa_tree ON taxa.parent_id = taxa_tree.taxon_id
JOIN (
  SELECT
      taxon_id,
      SUM(genomes) AS genomes,
      SUM(markers) AS loci,
      SUM(specimens) AS specimens,
      SUM(other) AS other,
      SUM(total_genomic) AS total_genomic
  FROM name_data_summaries
  JOIN taxon_names ON taxon_names.name_id = name_data_summaries.name_id
  GROUP BY taxon_id
) summaries ON taxa.id = summaries.taxon_id
LEFT JOIN (
  SELECT
    taxon_id,
    array_agg(name::text) filter (WHERE value_type = 'boolean') AS traits,
    jsonb_agg(CASE
     WHEN value_type = 'boolean' THEN jsonb_build_object('name', name, 'value', value_bool)
     WHEN value_type = 'string' THEN jsonb_build_object('name', name, 'value', value_str)
     WHEN value_type = 'integer' THEN jsonb_build_object('name', name, 'value', value_int)
     WHEN value_type = 'decimal' THEN jsonb_build_object('name', name, 'value', value_decimal)
     WHEN value_type = 'timestamp' THEN jsonb_build_object('name', name, 'value', value_timestamp)
    END) AS attributes
  FROM name_attributes
  JOIN taxon_names ON taxon_names.name_id = name_attributes.name_id
  GROUP BY taxon_id
) name_attributes ON taxa.id = name_attributes.taxon_id
LEFT JOIN (
  SELECT
    taxon_id,
    array_agg(DISTINCT vernacular_name) as names
  FROM vernacular_names
  JOIN taxon_names ON taxon_names.name_id = vernacular_names.name_id
  GROUP BY taxon_id
) vernacular_names ON taxa.id = vernacular_names.taxon_id;

CREATE UNIQUE INDEX IF NOT EXISTS species_id ON species (id);
CREATE INDEX IF NOT EXISTS species_dataset_id ON species (dataset_id);


-- Taxon classification
CREATE MATERIALIZED VIEW IF NOT EXISTS taxon_classification AS
SELECT
    taxon_id,
    array_agg(taxa_dag.canonical_name ORDER BY depth DESC) AS hierarchy,
    jsonb_object_agg(rank, canonical_name) AS ranks
FROM taxa_dag
GROUP BY taxon_id;

CREATE UNIQUE INDEX IF NOT EXISTS taxon_classification_taxon_id ON taxon_classification (taxon_id);


-- Statistics for all specimens
CREATE MATERIALIZED VIEW IF NOT EXISTS specimen_stats AS
SELECT DISTINCT
    specimens.id,
    SUM(CASE WHEN sequences.record_id IS NOT NULL THEN 1 ELSE 0 END) over (partition BY specimens.record_id ORDER BY specimens.record_id DESC) AS sequences,
    SUM(CASE WHEN annotation_events.representation IN ('Full', 'Partial') THEN 1 ELSE 0 END) over (partition BY specimens.record_id ORDER BY specimens.record_id DESC) AS whole_genomes,
    SUM(CASE WHEN sequencing_events.target_gene IS NOT NULL THEN 1 ELSE 0 END) over (partition BY specimens.record_id ORDER BY specimens.record_id DESC) AS markers
FROM specimens
LEFT JOIN subsamples ON subsamples.specimen_id = specimens.id
LEFT JOIN dna_extracts ON dna_extracts.subsample_id = subsamples.id
LEFT JOIN sequences ON sequences.dna_extract_id = dna_extracts.id
LEFT JOIN sequencing_events ON sequencing_events.sequence_id = sequences.id
LEFT JOIN annotation_events ON annotation_events.sequence_id = sequences.id;


-- Very high level stats for all data in the ARGA database
CREATE MATERIALIZED VIEW IF NOT EXISTS overview AS
SELECT 'data_type' AS category, 'sequences' AS name, count(*) AS total FROM sequences
UNION ALL SELECT 'data_type' AS category, 'whole_genomes' AS name, count(*) AS total FROM whole_genomes
UNION ALL SELECT 'data_type' AS category, 'loci' AS name, count(*) AS total FROM markers
UNION ALL SELECT 'data_type' AS category, 'specimens' AS name, count(*) AS total FROM specimens

UNION ALL

SELECT 'source' AS category, sources.name, count(distinct name_id) as total FROM sources
LEFT JOIN datasets ON source_id=sources.id
LEFT JOIN name_attributes ON datasets.id = name_attributes.dataset_id
GROUP BY sources.name

UNION ALL

SELECT 'dataset' AS category, datasets.name, count(*) AS total FROM name_attributes
JOIN datasets ON name_attributes.dataset_id = datasets.id
WHERE name_attributes.name = 'last_updated'
GROUP BY datasets.name
