use diesel::pg::Pg;
use diesel::prelude::*;

use arga_core::schema::classifications;
use arga_core::models::TaxonomicRank;
use diesel::sql_types::Bool;


type BoxedExpression<'a> = Box<dyn BoxableExpression<classifications::table, Pg, SqlType = Bool> + 'a>;


#[derive(Clone)]
pub enum FilterKind {
    Classification(Classification),
}

#[derive(Clone)]
pub enum Classification {
    Domain(String),
    Superkingdom(String),
    Kingdom(String),
    Subkingdom(String),
    Phylum(String),
    Subphylum(String),
    Superclass(String),
    Class(String),
    Subclass(String),
    Superorder(String),
    Order(String),
    Suborder(String),
    Superfamily(String),
    Family(String),
    Subfamily(String),
    Supertribe(String),
    Tribe(String),
    Subtribe(String),
    Genus(String),
    Subgenus(String),
    Species(String),
    Subspecies(String),
    Unranked(String),
    HigherTaxon(String),
    AggregateGenera(String),
    AggregateSpecies(String),
    Cohort(String),
    Division(String),
    IncertaeSedis(String),
    Infraclass(String),
    Infraorder(String),
    Section(String),
    Subdivision(String),
    Regnum(String),
    Familia(String),
    Classis(String),
    Ordo(String),
    Varietas(String),
    Forma(String),
    Subclassis(String),
    Superordo(String),
    Sectio(String),
    Nothovarietas(String),
    Subvarietas(String),
    Series(String),
    Infraspecies(String),
    Subfamilia(String),
    Subordo(String),
    Regio(String),
    SpecialForm(String),
}

#[derive(Clone)]
pub enum Filter {
    Include(FilterKind),
    Exclude(FilterKind),
}


pub fn filter_classifications(filters: &Vec<Filter>) -> classifications::BoxedQuery<Pg> {
    classifications::table
        .select(classifications::all_columns)
        .filter(with_filters(&filters).unwrap())
        .into_boxed()
}


/// Filter the classifications table with a global filter enum
pub fn with_filter(filter: &Filter) -> BoxedExpression {
    match filter {
        Filter::Include(kind) => match kind {
            FilterKind::Classification(value) => with_classification(value),
        }
        Filter::Exclude(kind) => match kind {
            FilterKind::Classification(value) => without_classification(value),
        }
    }
}

/// Narrow down the results from the classifications table with multiple filters
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

/// Filter the classifications table that belong to the provided classification
pub fn with_classification(classification: &Classification) -> BoxedExpression {
    use arga_core::schema::classifications::dsl::*;

    match classification {
        Classification::Domain(value) => Box::new(rank.eq(TaxonomicRank::Domain).and(canonical_name.eq(value))),
        Classification::Superkingdom(value) => Box::new(rank.eq(TaxonomicRank::Superkingdom).and(canonical_name.eq(value))),
        Classification::Kingdom(value) => Box::new(rank.eq(TaxonomicRank::Kingdom).and(canonical_name.eq(value))),
        Classification::Subkingdom(value) => Box::new(rank.eq(TaxonomicRank::Subkingdom).and(canonical_name.eq(value))),
        Classification::Phylum(value) => Box::new(rank.eq(TaxonomicRank::Phylum).and(canonical_name.eq(value))),
        Classification::Subphylum(value) => Box::new(rank.eq(TaxonomicRank::Subphylum).and(canonical_name.eq(value))),
        Classification::Superclass(value) => Box::new(rank.eq(TaxonomicRank::Superclass).and(canonical_name.eq(value))),
        Classification::Class(value) => Box::new(rank.eq(TaxonomicRank::Class).and(canonical_name.eq(value))),
        Classification::Subclass(value) => Box::new(rank.eq(TaxonomicRank::Subclass).and(canonical_name.eq(value))),
        Classification::Superorder(value) => Box::new(rank.eq(TaxonomicRank::Superorder).and(canonical_name.eq(value))),
        Classification::Order(value) => Box::new(rank.eq(TaxonomicRank::Order).and(canonical_name.eq(value))),
        Classification::Suborder(value) => Box::new(rank.eq(TaxonomicRank::Suborder).and(canonical_name.eq(value))),
        Classification::Superfamily(value) => Box::new(rank.eq(TaxonomicRank::Superfamily).and(canonical_name.eq(value))),
        Classification::Family(value) => Box::new(rank.eq(TaxonomicRank::Family).and(canonical_name.eq(value))),
        Classification::Subfamily(value) => Box::new(rank.eq(TaxonomicRank::Subfamily).and(canonical_name.eq(value))),
        Classification::Supertribe(value) => Box::new(rank.eq(TaxonomicRank::Supertribe).and(canonical_name.eq(value))),
        Classification::Tribe(value) => Box::new(rank.eq(TaxonomicRank::Tribe).and(canonical_name.eq(value))),
        Classification::Subtribe(value) => Box::new(rank.eq(TaxonomicRank::Subtribe).and(canonical_name.eq(value))),
        Classification::Genus(value) => Box::new(rank.eq(TaxonomicRank::Genus).and(canonical_name.eq(value))),
        Classification::Subgenus(value) => Box::new(rank.eq(TaxonomicRank::Subgenus).and(canonical_name.eq(value))),
        Classification::Species(value) => Box::new(rank.eq(TaxonomicRank::Species).and(canonical_name.eq(value))),
        Classification::Subspecies(value) => Box::new(rank.eq(TaxonomicRank::Subspecies).and(canonical_name.eq(value))),
        Classification::Unranked(value) => Box::new(rank.eq(TaxonomicRank::Unranked).and(canonical_name.eq(value))),
        Classification::HigherTaxon(value) => Box::new(rank.eq(TaxonomicRank::HigherTaxon).and(canonical_name.eq(value))),
        Classification::AggregateGenera(value) => Box::new(rank.eq(TaxonomicRank::AggregateGenera).and(canonical_name.eq(value))),
        Classification::AggregateSpecies(value) => Box::new(rank.eq(TaxonomicRank::AggregateSpecies).and(canonical_name.eq(value))),
        Classification::Cohort(value) => Box::new(rank.eq(TaxonomicRank::Cohort).and(canonical_name.eq(value))),
        Classification::Division(value) => Box::new(rank.eq(TaxonomicRank::Division).and(canonical_name.eq(value))),
        Classification::IncertaeSedis(value) => Box::new(rank.eq(TaxonomicRank::IncertaeSedis).and(canonical_name.eq(value))),
        Classification::Infraclass(value) => Box::new(rank.eq(TaxonomicRank::Infraclass).and(canonical_name.eq(value))),
        Classification::Infraorder(value) => Box::new(rank.eq(TaxonomicRank::Infraorder).and(canonical_name.eq(value))),
        Classification::Section(value) => Box::new(rank.eq(TaxonomicRank::Section).and(canonical_name.eq(value))),
        Classification::Subdivision(value) => Box::new(rank.eq(TaxonomicRank::Subdivision).and(canonical_name.eq(value))),
        Classification::Regnum(value) => Box::new(rank.eq(TaxonomicRank::Regnum).and(canonical_name.eq(value))),
        Classification::Familia(value) => Box::new(rank.eq(TaxonomicRank::Familia).and(canonical_name.eq(value))),
        Classification::Classis(value) => Box::new(rank.eq(TaxonomicRank::Classis).and(canonical_name.eq(value))),
        Classification::Ordo(value) => Box::new(rank.eq(TaxonomicRank::Ordo).and(canonical_name.eq(value))),
        Classification::Varietas(value) => Box::new(rank.eq(TaxonomicRank::Varietas).and(canonical_name.eq(value))),
        Classification::Forma(value) => Box::new(rank.eq(TaxonomicRank::Forma).and(canonical_name.eq(value))),
        Classification::Subclassis(value) => Box::new(rank.eq(TaxonomicRank::Subclassis).and(canonical_name.eq(value))),
        Classification::Superordo(value) => Box::new(rank.eq(TaxonomicRank::Superordo).and(canonical_name.eq(value))),
        Classification::Sectio(value) => Box::new(rank.eq(TaxonomicRank::Sectio).and(canonical_name.eq(value))),
        Classification::Nothovarietas(value) => Box::new(rank.eq(TaxonomicRank::Nothovarietas).and(canonical_name.eq(value))),
        Classification::Subvarietas(value) => Box::new(rank.eq(TaxonomicRank::Subvarietas).and(canonical_name.eq(value))),
        Classification::Series(value) => Box::new(rank.eq(TaxonomicRank::Series).and(canonical_name.eq(value))),
        Classification::Infraspecies(value) => Box::new(rank.eq(TaxonomicRank::Infraspecies).and(canonical_name.eq(value))),
        Classification::Subfamilia(value) => Box::new(rank.eq(TaxonomicRank::Subfamilia).and(canonical_name.eq(value))),
        Classification::Subordo(value) => Box::new(rank.eq(TaxonomicRank::Subordo).and(canonical_name.eq(value))),
        Classification::Regio(value) => Box::new(rank.eq(TaxonomicRank::Regio).and(canonical_name.eq(value))),
        Classification::SpecialForm(value) => Box::new(rank.eq(TaxonomicRank::SpecialForm).and(canonical_name.eq(value))),
    }
}

/// Filter the classifications table that do not belong to the provided classification
pub fn without_classification(classification: &Classification) -> BoxedExpression {
    use arga_core::schema::classifications::dsl::*;

    match classification {
        Classification::Domain(value) => Box::new(rank.eq(TaxonomicRank::Domain).and(canonical_name.ne(value))),
        Classification::Superkingdom(value) => Box::new(rank.eq(TaxonomicRank::Superkingdom).and(canonical_name.ne(value))),
        Classification::Kingdom(value) => Box::new(rank.eq(TaxonomicRank::Kingdom).and(canonical_name.ne(value))),
        Classification::Subkingdom(value) => Box::new(rank.eq(TaxonomicRank::Subkingdom).and(canonical_name.ne(value))),
        Classification::Phylum(value) => Box::new(rank.eq(TaxonomicRank::Phylum).and(canonical_name.ne(value))),
        Classification::Subphylum(value) => Box::new(rank.eq(TaxonomicRank::Subphylum).and(canonical_name.ne(value))),
        Classification::Superclass(value) => Box::new(rank.eq(TaxonomicRank::Superclass).and(canonical_name.ne(value))),
        Classification::Class(value) => Box::new(rank.eq(TaxonomicRank::Class).and(canonical_name.ne(value))),
        Classification::Subclass(value) => Box::new(rank.eq(TaxonomicRank::Subclass).and(canonical_name.ne(value))),
        Classification::Superorder(value) => Box::new(rank.eq(TaxonomicRank::Superorder).and(canonical_name.ne(value))),
        Classification::Order(value) => Box::new(rank.eq(TaxonomicRank::Order).and(canonical_name.ne(value))),
        Classification::Suborder(value) => Box::new(rank.eq(TaxonomicRank::Suborder).and(canonical_name.ne(value))),
        Classification::Superfamily(value) => Box::new(rank.eq(TaxonomicRank::Superfamily).and(canonical_name.ne(value))),
        Classification::Family(value) => Box::new(rank.eq(TaxonomicRank::Family).and(canonical_name.ne(value))),
        Classification::Subfamily(value) => Box::new(rank.eq(TaxonomicRank::Subfamily).and(canonical_name.ne(value))),
        Classification::Supertribe(value) => Box::new(rank.eq(TaxonomicRank::Supertribe).and(canonical_name.ne(value))),
        Classification::Tribe(value) => Box::new(rank.eq(TaxonomicRank::Tribe).and(canonical_name.ne(value))),
        Classification::Subtribe(value) => Box::new(rank.eq(TaxonomicRank::Subtribe).and(canonical_name.ne(value))),
        Classification::Genus(value) => Box::new(rank.eq(TaxonomicRank::Genus).and(canonical_name.ne(value))),
        Classification::Subgenus(value) => Box::new(rank.eq(TaxonomicRank::Subgenus).and(canonical_name.ne(value))),
        Classification::Species(value) => Box::new(rank.eq(TaxonomicRank::Species).and(canonical_name.ne(value))),
        Classification::Subspecies(value) => Box::new(rank.eq(TaxonomicRank::Subspecies).and(canonical_name.ne(value))),
        Classification::Unranked(value) => Box::new(rank.eq(TaxonomicRank::Unranked).and(canonical_name.ne(value))),
        Classification::HigherTaxon(value) => Box::new(rank.eq(TaxonomicRank::HigherTaxon).and(canonical_name.ne(value))),
        Classification::AggregateGenera(value) => Box::new(rank.eq(TaxonomicRank::AggregateGenera).and(canonical_name.ne(value))),
        Classification::AggregateSpecies(value) => Box::new(rank.eq(TaxonomicRank::AggregateSpecies).and(canonical_name.ne(value))),
        Classification::Cohort(value) => Box::new(rank.eq(TaxonomicRank::Cohort).and(canonical_name.ne(value))),
        Classification::Division(value) => Box::new(rank.eq(TaxonomicRank::Division).and(canonical_name.ne(value))),
        Classification::IncertaeSedis(value) => Box::new(rank.eq(TaxonomicRank::IncertaeSedis).and(canonical_name.ne(value))),
        Classification::Infraclass(value) => Box::new(rank.eq(TaxonomicRank::Infraclass).and(canonical_name.ne(value))),
        Classification::Infraorder(value) => Box::new(rank.eq(TaxonomicRank::Infraorder).and(canonical_name.ne(value))),
        Classification::Section(value) => Box::new(rank.eq(TaxonomicRank::Section).and(canonical_name.ne(value))),
        Classification::Subdivision(value) => Box::new(rank.eq(TaxonomicRank::Subdivision).and(canonical_name.ne(value))),
        Classification::Regnum(value) => Box::new(rank.eq(TaxonomicRank::Regnum).and(canonical_name.ne(value))),
        Classification::Familia(value) => Box::new(rank.eq(TaxonomicRank::Familia).and(canonical_name.ne(value))),
        Classification::Classis(value) => Box::new(rank.eq(TaxonomicRank::Classis).and(canonical_name.ne(value))),
        Classification::Ordo(value) => Box::new(rank.eq(TaxonomicRank::Ordo).and(canonical_name.ne(value))),
        Classification::Varietas(value) => Box::new(rank.eq(TaxonomicRank::Varietas).and(canonical_name.ne(value))),
        Classification::Forma(value) => Box::new(rank.eq(TaxonomicRank::Forma).and(canonical_name.ne(value))),
        Classification::Subclassis(value) => Box::new(rank.eq(TaxonomicRank::Subclassis).and(canonical_name.ne(value))),
        Classification::Superordo(value) => Box::new(rank.eq(TaxonomicRank::Superordo).and(canonical_name.ne(value))),
        Classification::Sectio(value) => Box::new(rank.eq(TaxonomicRank::Sectio).and(canonical_name.ne(value))),
        Classification::Nothovarietas(value) => Box::new(rank.eq(TaxonomicRank::Nothovarietas).and(canonical_name.ne(value))),
        Classification::Subvarietas(value) => Box::new(rank.eq(TaxonomicRank::Subvarietas).and(canonical_name.ne(value))),
        Classification::Series(value) => Box::new(rank.eq(TaxonomicRank::Series).and(canonical_name.ne(value))),
        Classification::Infraspecies(value) => Box::new(rank.eq(TaxonomicRank::Infraspecies).and(canonical_name.ne(value))),
        Classification::Subfamilia(value) => Box::new(rank.eq(TaxonomicRank::Subfamilia).and(canonical_name.ne(value))),
        Classification::Subordo(value) => Box::new(rank.eq(TaxonomicRank::Subordo).and(canonical_name.ne(value))),
        Classification::Regio(value) => Box::new(rank.eq(TaxonomicRank::Regio).and(canonical_name.ne(value))),
        Classification::SpecialForm(value) => Box::new(rank.eq(TaxonomicRank::SpecialForm).and(canonical_name.ne(value))),
    }
}
