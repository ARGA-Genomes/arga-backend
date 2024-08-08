use arga_core::models::TaxonomicRank;
use arga_core::schema::taxa;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;


type BoxedExpression<'a> = Box<dyn BoxableExpression<taxa::table, Pg, SqlType = Bool> + 'a>;


#[derive(Clone)]
pub enum FilterKind {
    Classification(Classification),
}

#[derive(Clone, Debug)]
pub enum Classification {
    Domain(String),
    Superkingdom(String),
    Kingdom(String),
    Subkingdom(String),
    Infrakingdom(String),
    Superphylum(String),
    Phylum(String),
    Subphylum(String),
    Infraphylum(String),
    Parvphylum(String),
    Gigaclass(String),
    Megaclass(String),
    Superclass(String),
    Class(String),
    Subclass(String),
    Infraclass(String),
    Subterclass(String),
    Superorder(String),
    Order(String),
    Hyporder(String),
    Minorder(String),
    Suborder(String),
    Infraorder(String),
    Parvorder(String),
    Epifamily(String),
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
    Variety(String),
    Subvariety(String),
    Natio(String),
    Mutatio(String),
    Unranked(String),
    HigherTaxon(String),
    AggregateGenera(String),
    AggregateSpecies(String),
    Cohort(String),
    Subcohort(String),
    Division(String),
    IncertaeSedis(String),
    Infragenus(String),
    Section(String),
    Subsection(String),
    Subdivision(String),
    Regnum(String),
    Familia(String),
    Classis(String),
    Ordo(String),
    Varietas(String),
    Forma(String),
    Subforma(String),
    Subclassis(String),
    Superordo(String),
    Sectio(String),
    Subsectio(String),
    Nothovarietas(String),
    Subvarietas(String),
    Series(String),
    Subseries(String),
    Superspecies(String),
    Infraspecies(String),
    Subfamilia(String),
    Subordo(String),
    Regio(String),
    SpecialForm(String),
    Pathovar(String),
    Serovar(String),
    Biovar(String),
}

#[derive(Clone)]
pub enum Filter {
    Include(FilterKind),
    Exclude(FilterKind),
}


pub fn filter_classifications(filters: &Vec<Filter>) -> taxa::BoxedQuery<Pg> {
    taxa::table
        .select(taxa::all_columns)
        .filter(with_filters(&filters).unwrap())
        .into_boxed()
}


/// Filter the classifications table with a global filter enum
pub fn with_filter(filter: &Filter) -> BoxedExpression {
    match filter {
        Filter::Include(kind) => match kind {
            FilterKind::Classification(value) => with_classification(value),
        },
        Filter::Exclude(kind) => match kind {
            FilterKind::Classification(value) => without_classification(value),
        },
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


pub fn decompose_classification(classification: &Classification) -> (TaxonomicRank, String) {
    match classification {
        Classification::Domain(value) => (TaxonomicRank::Domain, value.clone()),
        Classification::Superkingdom(value) => (TaxonomicRank::Superkingdom, value.clone()),
        Classification::Kingdom(value) => (TaxonomicRank::Kingdom, value.clone()),
        Classification::Subkingdom(value) => (TaxonomicRank::Subkingdom, value.clone()),
        Classification::Infrakingdom(value) => (TaxonomicRank::Infrakingdom, value.clone()),
        Classification::Superphylum(value) => (TaxonomicRank::Superphylum, value.clone()),
        Classification::Phylum(value) => (TaxonomicRank::Phylum, value.clone()),
        Classification::Subphylum(value) => (TaxonomicRank::Subphylum, value.clone()),
        Classification::Infraphylum(value) => (TaxonomicRank::Infraphylum, value.clone()),
        Classification::Parvphylum(value) => (TaxonomicRank::Parvphylum, value.clone()),
        Classification::Gigaclass(value) => (TaxonomicRank::Gigaclass, value.clone()),
        Classification::Megaclass(value) => (TaxonomicRank::Megaclass, value.clone()),
        Classification::Superclass(value) => (TaxonomicRank::Superclass, value.clone()),
        Classification::Class(value) => (TaxonomicRank::Class, value.clone()),
        Classification::Subclass(value) => (TaxonomicRank::Subclass, value.clone()),
        Classification::Infraclass(value) => (TaxonomicRank::Infraclass, value.clone()),
        Classification::Subterclass(value) => (TaxonomicRank::Subterclass, value.clone()),
        Classification::Superorder(value) => (TaxonomicRank::Superorder, value.clone()),
        Classification::Order(value) => (TaxonomicRank::Order, value.clone()),
        Classification::Hyporder(value) => (TaxonomicRank::Hyporder, value.clone()),
        Classification::Minorder(value) => (TaxonomicRank::Minorder, value.clone()),
        Classification::Suborder(value) => (TaxonomicRank::Suborder, value.clone()),
        Classification::Infraorder(value) => (TaxonomicRank::Infraorder, value.clone()),
        Classification::Parvorder(value) => (TaxonomicRank::Parvorder, value.clone()),
        Classification::Epifamily(value) => (TaxonomicRank::Epifamily, value.clone()),
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
        Classification::Variety(value) => (TaxonomicRank::Variety, value.clone()),
        Classification::Subvariety(value) => (TaxonomicRank::Subvariety, value.clone()),
        Classification::Natio(value) => (TaxonomicRank::Natio, value.clone()),
        Classification::Mutatio(value) => (TaxonomicRank::Mutatio, value.clone()),
        Classification::Unranked(value) => (TaxonomicRank::Unranked, value.clone()),
        Classification::HigherTaxon(value) => (TaxonomicRank::HigherTaxon, value.clone()),
        Classification::AggregateGenera(value) => (TaxonomicRank::AggregateGenera, value.clone()),
        Classification::AggregateSpecies(value) => (TaxonomicRank::AggregateSpecies, value.clone()),
        Classification::Cohort(value) => (TaxonomicRank::Cohort, value.clone()),
        Classification::Subcohort(value) => (TaxonomicRank::Subcohort, value.clone()),
        Classification::Division(value) => (TaxonomicRank::Division, value.clone()),
        Classification::IncertaeSedis(value) => (TaxonomicRank::IncertaeSedis, value.clone()),
        Classification::Infragenus(value) => (TaxonomicRank::Infragenus, value.clone()),
        Classification::Section(value) => (TaxonomicRank::Section, value.clone()),
        Classification::Subsection(value) => (TaxonomicRank::Subsection, value.clone()),
        Classification::Subdivision(value) => (TaxonomicRank::Subdivision, value.clone()),
        Classification::Regnum(value) => (TaxonomicRank::Regnum, value.clone()),
        Classification::Familia(value) => (TaxonomicRank::Familia, value.clone()),
        Classification::Classis(value) => (TaxonomicRank::Classis, value.clone()),
        Classification::Ordo(value) => (TaxonomicRank::Ordo, value.clone()),
        Classification::Varietas(value) => (TaxonomicRank::Varietas, value.clone()),
        Classification::Forma(value) => (TaxonomicRank::Forma, value.clone()),
        Classification::Subforma(value) => (TaxonomicRank::Subforma, value.clone()),
        Classification::Subclassis(value) => (TaxonomicRank::Subclassis, value.clone()),
        Classification::Superordo(value) => (TaxonomicRank::Superordo, value.clone()),
        Classification::Sectio(value) => (TaxonomicRank::Sectio, value.clone()),
        Classification::Subsectio(value) => (TaxonomicRank::Subsectio, value.clone()),
        Classification::Nothovarietas(value) => (TaxonomicRank::Nothovarietas, value.clone()),
        Classification::Subvarietas(value) => (TaxonomicRank::Subvarietas, value.clone()),
        Classification::Series(value) => (TaxonomicRank::Series, value.clone()),
        Classification::Subseries(value) => (TaxonomicRank::Subseries, value.clone()),
        Classification::Superspecies(value) => (TaxonomicRank::Superspecies, value.clone()),
        Classification::Infraspecies(value) => (TaxonomicRank::Infraspecies, value.clone()),
        Classification::Subfamilia(value) => (TaxonomicRank::Subfamilia, value.clone()),
        Classification::Subordo(value) => (TaxonomicRank::Subordo, value.clone()),
        Classification::Regio(value) => (TaxonomicRank::Regio, value.clone()),
        Classification::SpecialForm(value) => (TaxonomicRank::SpecialForm, value.clone()),
        Classification::Pathovar(value) => (TaxonomicRank::Pathovar, value.clone()),
        Classification::Serovar(value) => (TaxonomicRank::Serovar, value.clone()),
        Classification::Biovar(value) => (TaxonomicRank::Biovar, value.clone()),
    }
}


/// Filter the classifications table that belong to the provided classification
pub fn with_classification(classification: &Classification) -> BoxedExpression {
    use arga_core::schema::taxa::dsl::*;
    let (taxon_rank, value) = decompose_classification(classification);
    Box::new(rank.eq(taxon_rank).and(canonical_name.eq(value)))
}

/// Filter the classifications table that do not belong to the provided classification
pub fn without_classification(classification: &Classification) -> BoxedExpression {
    use arga_core::schema::taxa::dsl::*;
    let (taxon_rank, value) = decompose_classification(classification);
    Box::new(rank.eq(taxon_rank).and(canonical_name.ne(value)))
}
