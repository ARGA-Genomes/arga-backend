use diesel::pg::Pg;
use diesel::prelude::*;

use arga_core::schema_gnl::classification_species;
use arga_core::models::TaxonomicRank;
use diesel::sql_types::Bool;

use super::classification_filters::Classification;


type BoxedExpression<'a> = Box<dyn BoxableExpression<classification_species::table, Pg, SqlType = Bool> + 'a>;


#[derive(Clone)]
pub enum FilterKind {
    Classification(Classification),
    ParentClassification(Classification),
}

#[derive(Clone)]
pub enum Filter {
    Include(FilterKind),
    Exclude(FilterKind),
}


/// Filter the classification species view with a global filter enum
pub fn with_filter(filter: &Filter) -> BoxedExpression {
    match filter {
        Filter::Include(kind) => match kind {
            FilterKind::Classification(value) => with_classification(value),
            FilterKind::ParentClassification(value) => with_parent_classification(value),
        }
        Filter::Exclude(kind) => match kind {
            FilterKind::Classification(value) => without_classification(value),
            FilterKind::ParentClassification(value) => without_parent_classification(value),
        }
    }
}

/// Narrow down the results from the classification species view with multiple filters
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

pub fn decompose_classification(classification: &Classification) -> (TaxonomicRank, String) {
    match classification {
        Classification::Domain(value) => (TaxonomicRank::Domain, value.clone()),
        Classification::Superkingdom(value) => (TaxonomicRank::Superkingdom, value.clone()),
        Classification::Kingdom(value) => (TaxonomicRank::Kingdom, value.clone()),
        Classification::Subkingdom(value) => (TaxonomicRank::Subkingdom, value.clone()),
        Classification::Phylum(value) => (TaxonomicRank::Phylum, value.clone()),
        Classification::Subphylum(value) => (TaxonomicRank::Subphylum, value.clone()),
        Classification::Superclass(value) => (TaxonomicRank::Superclass, value.clone()),
        Classification::Class(value) => (TaxonomicRank::Class, value.clone()),
        Classification::Subclass(value) => (TaxonomicRank::Subclass, value.clone()),
        Classification::Superorder(value) => (TaxonomicRank::Superorder, value.clone()),
        Classification::Order(value) => (TaxonomicRank::Order, value.clone()),
        Classification::Suborder(value) => (TaxonomicRank::Suborder, value.clone()),
        Classification::Superfamily(value) => (TaxonomicRank::Superfamily, value.clone()),
        Classification::Family(value) => (TaxonomicRank::Family, value.clone()),
        Classification::Subfamily(value) => (TaxonomicRank::Subfamily, value.clone()),
        Classification::Supertribe(value) => (TaxonomicRank::Supertribe, value.clone()),
        Classification::Tribe(value) => (TaxonomicRank::Tribe, value.clone()),
        Classification::Subtribe(value) => (TaxonomicRank::Subtribe, value.clone()),
        Classification::Genus(value) => (TaxonomicRank::Genus, value.clone()),
        Classification::Subgenus(value) => (TaxonomicRank::Subgenus, value.clone()),
        Classification::Species(value) => (TaxonomicRank::Species, value.clone()),
        Classification::Subspecies(value) => (TaxonomicRank::Subspecies, value.clone()),
        Classification::Unranked(value) => (TaxonomicRank::Unranked, value.clone()),
        Classification::HigherTaxon(value) => (TaxonomicRank::HigherTaxon, value.clone()),
        Classification::AggregateGenera(value) => (TaxonomicRank::AggregateGenera, value.clone()),
        Classification::AggregateSpecies(value) => (TaxonomicRank::AggregateSpecies, value.clone()),
        Classification::Cohort(value) => (TaxonomicRank::Cohort, value.clone()),
        Classification::Division(value) => (TaxonomicRank::Division, value.clone()),
        Classification::IncertaeSedis(value) => (TaxonomicRank::IncertaeSedis, value.clone()),
        Classification::Infraclass(value) => (TaxonomicRank::Infraclass, value.clone()),
        Classification::Infraorder(value) => (TaxonomicRank::Infraorder, value.clone()),
        Classification::Section(value) => (TaxonomicRank::Section, value.clone()),
        Classification::Subdivision(value) => (TaxonomicRank::Subdivision, value.clone()),
        Classification::Regnum(value) => (TaxonomicRank::Regnum, value.clone()),
        Classification::Familia(value) => (TaxonomicRank::Familia, value.clone()),
        Classification::Classis(value) => (TaxonomicRank::Classis, value.clone()),
        Classification::Ordo(value) => (TaxonomicRank::Ordo, value.clone()),
        Classification::Varietas(value) => (TaxonomicRank::Varietas, value.clone()),
        Classification::Forma(value) => (TaxonomicRank::Forma, value.clone()),
        Classification::Subclassis(value) => (TaxonomicRank::Subclassis, value.clone()),
        Classification::Superordo(value) => (TaxonomicRank::Superordo, value.clone()),
        Classification::Sectio(value) => (TaxonomicRank::Sectio, value.clone()),
        Classification::Nothovarietas(value) => (TaxonomicRank::Nothovarietas, value.clone()),
        Classification::Subvarietas(value) => (TaxonomicRank::Subvarietas, value.clone()),
        Classification::Series(value) => (TaxonomicRank::Series, value.clone()),
        Classification::Infraspecies(value) => (TaxonomicRank::Infraspecies, value.clone()),
        Classification::Subfamilia(value) => (TaxonomicRank::Subfamilia, value.clone()),
        Classification::Subordo(value) => (TaxonomicRank::Subordo, value.clone()),
        Classification::Regio(value) => (TaxonomicRank::Regio, value.clone()),
        Classification::SpecialForm(value) => (TaxonomicRank::SpecialForm, value.clone()),
    }
}

/// Filter the classifications table that belong to the provided classification
pub fn with_classification(classification: &Classification) -> BoxedExpression {
    use classification_species::classification_rank as rank;
    use classification_species::classification_canonical_name as name;

    let (taxon_rank, value) = decompose_classification(classification);
    Box::new(rank.eq(taxon_rank).and(name.eq(value)))
}

/// Filter the classifications table that do not belong to the provided classification
pub fn without_classification(classification: &Classification) -> BoxedExpression {
    use classification_species::classification_rank as rank;
    use classification_species::classification_canonical_name as name;

    let (taxon_rank, value) = decompose_classification(classification);
    Box::new(rank.eq(taxon_rank).and(name.ne(value)))
}

/// Filter the classifications table that belong to the provided classification
pub fn with_parent_classification(classification: &Classification) -> BoxedExpression {
    use classification_species::parent_rank as rank;
    use classification_species::parent_canonical_name as name;

    let (taxon_rank, value) = decompose_classification(classification);
    Box::new(rank.eq(taxon_rank).and(name.eq(value)))
}

/// Filter the classifications table that do not belong to the provided classification
pub fn without_parent_classification(classification: &Classification) -> BoxedExpression {
    use classification_species::parent_rank as rank;
    use classification_species::parent_canonical_name as name;

    let (taxon_rank, value) = decompose_classification(classification);
    Box::new(rank.eq(taxon_rank).and(name.ne(value)))
}
