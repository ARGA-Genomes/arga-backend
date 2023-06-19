CREATE TABLE assembly_stats (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    assembly_id uuid REFERENCES assemblies NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp,
    updated_at timestamp with time zone NOT NULL DEFAULT current_timestamp,

    total_length int,
    spanned_gaps int,
    unspanned_gaps int,
    region_count int,
    scaffold_count int,
    scaffold_N50 int,
    scaffold_L50 int,
    scaffold_N75 int,
    scaffold_N90 int,
    contig_count int,
    contig_N50 int,
    contig_L50 int,
    total_gap_length int,
    molecule_count int,
    top_level_count int,
    component_count int,
    gc_perc int
);
