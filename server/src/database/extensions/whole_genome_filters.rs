use diesel::pg::Pg;
use diesel::prelude::*;

use arga_core::schema_gnl::whole_genomes;
use diesel::sql_types::{Bool, Nullable};


#[derive(Clone)]
pub enum FilterKind {
    AssemblyLevel(AssemblyLevel),
    GenomeRepresentation(GenomeRepresentation),
    ReleaseType(ReleaseType),
}

#[derive(Clone)]
pub enum AssemblyLevel {
    CompleteGenome,
    Chromosome,
    Scaffold,
    Contig,
}

#[derive(Clone)]
pub enum GenomeRepresentation {
    Complete,
    Full,
    Partial,
}

#[derive(Clone)]
pub enum ReleaseType {
    Major,
    Minor,
    Patch,
}

#[derive(Clone)]
pub enum Filter {
    Include(FilterKind),
    Exclude(FilterKind),
}


pub fn filter_whole_genomes(filters: &Vec<Filter>) -> whole_genomes::BoxedQuery<Pg> {
    whole_genomes::table
        .select(whole_genomes::all_columns)
        .filter(with_filters(&filters).unwrap())
        .into_boxed()
}


type BoxedExpression<'a> = Box<dyn BoxableExpression<whole_genomes::table, Pg, SqlType = Nullable<Bool>> + 'a>;


/// Filter the taxa table with a global filter enum
pub fn with_filter(filter: &Filter) -> BoxedExpression {
    match filter {
        Filter::Include(kind) => match kind {
            FilterKind::AssemblyLevel(value) => with_assembly_level(value),
            FilterKind::GenomeRepresentation(value) => with_genome_representation(value),
            FilterKind::ReleaseType(value) => with_release_type(value),
        }
        Filter::Exclude(kind) => match kind {
            FilterKind::AssemblyLevel(value) => without_assembly_level(value),
            FilterKind::GenomeRepresentation(value) => without_genome_representation(value),
            FilterKind::ReleaseType(value) => without_release_type(value),
        }
    }
}

/// Narrow down the results from the whole genome table with multiple filters
pub fn with_filters(filters: &Vec<Filter>) -> Option<BoxedExpression> {
    let mut predicates: Option<BoxedExpression> = None;

    for filter in filters {
        let predicate = with_filter(filter);

        predicates = match predicates {
            None => Some(predicate),
            Some(others) => Some(Box::new(others.and(predicate))),
        }
    }

    predicates
}


/// Filter the whole genomes table with an assembly level
pub fn with_assembly_level(value: &AssemblyLevel) -> BoxedExpression {
    match value {
        AssemblyLevel::CompleteGenome => Box::new(whole_genomes::quality.eq("Complete Genome")),
        AssemblyLevel::Chromosome => Box::new(whole_genomes::quality.eq("Chromosome")),
        AssemblyLevel::Scaffold => Box::new(whole_genomes::quality.eq("Scaffold")),
        AssemblyLevel::Contig => Box::new(whole_genomes::quality.eq("Contig")),
    }
}

pub fn without_assembly_level(value: &AssemblyLevel) -> BoxedExpression {
    match value {
        AssemblyLevel::CompleteGenome => Box::new(whole_genomes::quality.ne("Complete Genome")),
        AssemblyLevel::Chromosome => Box::new(whole_genomes::quality.ne("Chromosome")),
        AssemblyLevel::Scaffold => Box::new(whole_genomes::quality.ne("Scaffold")),
        AssemblyLevel::Contig => Box::new(whole_genomes::quality.ne("Contig")),
    }
}


/// Filter the whole genomes table with a genome representation
pub fn with_genome_representation(value: &GenomeRepresentation) -> BoxedExpression {
    match value {
        GenomeRepresentation::Complete => Box::new(whole_genomes::representation.eq("Complete")),
        GenomeRepresentation::Full => Box::new(whole_genomes::representation.eq("Full")),
        GenomeRepresentation::Partial => Box::new(whole_genomes::representation.eq("Partial")),
    }
}

pub fn without_genome_representation(value: &GenomeRepresentation) -> BoxedExpression {
    match value {
        GenomeRepresentation::Complete => Box::new(whole_genomes::representation.ne("Complete")),
        GenomeRepresentation::Full => Box::new(whole_genomes::representation.ne("Full")),
        GenomeRepresentation::Partial => Box::new(whole_genomes::representation.ne("Partial")),
    }
}


/// Filter the whole genomes table with a release type
pub fn with_release_type(value: &ReleaseType) -> BoxedExpression {
    match value {
        ReleaseType::Major => Box::new(whole_genomes::release_type.eq("Major")),
        ReleaseType::Minor => Box::new(whole_genomes::release_type.eq("Minor")),
        ReleaseType::Patch => Box::new(whole_genomes::release_type.eq("Patch")),
    }
}

pub fn without_release_type(value: &ReleaseType) -> BoxedExpression {
    match value {
        ReleaseType::Major => Box::new(whole_genomes::release_type.ne("Major")),
        ReleaseType::Minor => Box::new(whole_genomes::release_type.ne("Minor")),
        ReleaseType::Patch => Box::new(whole_genomes::release_type.ne("Patch")),
    }
}
