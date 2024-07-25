use arga_core::search::SearchFilter;
use async_graphql::{from_value, Enum, InputObject, Value};
use serde::{Deserialize, Serialize};

use super::search::SearchDataType;
use super::species::DataType;
use super::taxonomy::TaxonomicVernacularGroup;
use super::whole_genomes::{AssemblyLevel, GenomeRepresentation, ReleaseType};
use crate::database::extensions::classification_filters::Classification;
use crate::database::extensions::filters::{Filter, FilterKind};
use crate::database::extensions::whole_genome_filters::{
    Filter as WholeGenomeFilter,
    FilterKind as WholeGenomeFilterKind,
};
use crate::http::Error;


#[derive(Clone, Debug, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum FilterType {
    VernacularGroup,
    HasData,
    // Ecology,
    // Ibra,
    // Imcra,
    // State,
    // DrainageBasin,

    // BushfireRecovery,

    // classification ranks
    Domain,
    Superkingdom,
    Kingdom,
    Subkingdom,
    Phylum,
    Subphylum,
    Superclass,
    Class,
    Subclass,
    Superorder,
    Order,
    Suborder,
    Hyporder,
    Superfamily,
    Family,
    Subfamily,
    Supertribe,
    Tribe,
    Subtribe,
    Genus,
    Subgenus,
    Cohort,
    Subcohort,
    Division,
    Section,
    Subdivision,
    Regnum,
    Familia,
    Classis,
    Ordo,
    Forma,
    Subclassis,
    Superordo,
    Sectio,
    Series,
    Subfamilia,
    Subordo,
    Regio,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum FilterAction {
    Include,
    Exclude,
}

/// An all purpose filter to apply to queries for species.
#[derive(Clone, Debug, Serialize, Deserialize, InputObject)]
pub struct FilterItem {
    filter: FilterType,
    action: FilterAction,
    value: String,
}

/// Converts a graphql filter into the common filter enum
impl TryFrom<FilterItem> for Filter {
    type Error = Error;

    fn try_from(source: FilterItem) -> Result<Self, Self::Error> {
        let kind = match source.filter {
            FilterType::VernacularGroup => {
                FilterKind::VernacularGroup(from_value::<TaxonomicVernacularGroup>(Value::String(source.value))?.into())
            }

            FilterType::HasData => FilterKind::HasData(from_value::<DataType>(Value::String(source.value))?.into()),

            // FilterType::Ecology => FilterKind::Ecology(source.value),
            // FilterType::Ibra => FilterKind::Ibra(source.value),
            // FilterType::Imcra => FilterKind::Imcra(source.value),
            // FilterType::State => FilterKind::State(source.value),
            // FilterType::DrainageBasin => FilterKind::DrainageBasin(source.value),

            // FilterType::BushfireRecovery => FilterKind::BushfireRecovery(
            //     from_value::<BushfireRecoveryTrait>(Value::String(source.value))?.into()
            // ),
            FilterType::Domain => FilterKind::Classification(Classification::Domain(source.value)),
            FilterType::Superkingdom => FilterKind::Classification(Classification::Superkingdom(source.value)),
            FilterType::Kingdom => FilterKind::Classification(Classification::Kingdom(source.value)),
            FilterType::Subkingdom => FilterKind::Classification(Classification::Subkingdom(source.value)),
            FilterType::Phylum => FilterKind::Classification(Classification::Phylum(source.value)),
            FilterType::Subphylum => FilterKind::Classification(Classification::Subphylum(source.value)),
            FilterType::Superclass => FilterKind::Classification(Classification::Superclass(source.value)),
            FilterType::Class => FilterKind::Classification(Classification::Class(source.value)),
            FilterType::Subclass => FilterKind::Classification(Classification::Subclass(source.value)),
            FilterType::Superorder => FilterKind::Classification(Classification::Superorder(source.value)),
            FilterType::Order => FilterKind::Classification(Classification::Order(source.value)),
            FilterType::Suborder => FilterKind::Classification(Classification::Suborder(source.value)),
            FilterType::Hyporder => FilterKind::Classification(Classification::Hyporder(source.value)),
            FilterType::Superfamily => FilterKind::Classification(Classification::Superfamily(source.value)),
            FilterType::Family => FilterKind::Classification(Classification::Family(source.value)),
            FilterType::Subfamily => FilterKind::Classification(Classification::Subfamily(source.value)),
            FilterType::Supertribe => FilterKind::Classification(Classification::Supertribe(source.value)),
            FilterType::Tribe => FilterKind::Classification(Classification::Tribe(source.value)),
            FilterType::Subtribe => FilterKind::Classification(Classification::Subtribe(source.value)),
            FilterType::Genus => FilterKind::Classification(Classification::Genus(source.value)),
            FilterType::Subgenus => FilterKind::Classification(Classification::Subgenus(source.value)),
            FilterType::Cohort => FilterKind::Classification(Classification::Cohort(source.value)),
            FilterType::Subcohort => FilterKind::Classification(Classification::Subcohort(source.value)),
            FilterType::Division => FilterKind::Classification(Classification::Division(source.value)),
            FilterType::Subdivision => FilterKind::Classification(Classification::Subdivision(source.value)),
            FilterType::Section => FilterKind::Classification(Classification::Section(source.value)),
            FilterType::Regnum => FilterKind::Classification(Classification::Regnum(source.value)),
            FilterType::Familia => FilterKind::Classification(Classification::Familia(source.value)),
            FilterType::Classis => FilterKind::Classification(Classification::Classis(source.value)),
            FilterType::Ordo => FilterKind::Classification(Classification::Ordo(source.value)),
            FilterType::Forma => FilterKind::Classification(Classification::Forma(source.value)),
            FilterType::Subclassis => FilterKind::Classification(Classification::Subclassis(source.value)),
            FilterType::Superordo => FilterKind::Classification(Classification::Superordo(source.value)),
            FilterType::Sectio => FilterKind::Classification(Classification::Sectio(source.value)),
            FilterType::Series => FilterKind::Classification(Classification::Series(source.value)),
            FilterType::Subfamilia => FilterKind::Classification(Classification::Subfamilia(source.value)),
            FilterType::Subordo => FilterKind::Classification(Classification::Subordo(source.value)),
            FilterType::Regio => FilterKind::Classification(Classification::Regio(source.value)),
        };

        Ok(match source.action {
            FilterAction::Include => Filter::Include(kind),
            FilterAction::Exclude => Filter::Exclude(kind),
        })
    }
}


pub fn convert_filters(items: Vec<FilterItem>) -> Result<Vec<Filter>, Error> {
    let mut filters = Vec::new();
    for item in items {
        filters.push(item.try_into()?);
    }
    Ok(filters)
}


/// An all purpose filter to apply to queries for whole genomes.
#[derive(Clone, Debug, Serialize, Deserialize, InputObject)]
pub struct WholeGenomeFilterItem {
    filter: WholeGenomeFilterType,
    action: FilterAction,
    value: String,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum WholeGenomeFilterType {
    AssemblyLevel,
    GenomeRepresentation,
    ReleaseType,
}

/// Converts a graphql whole genome filter into the whole genome filter enum
impl TryFrom<WholeGenomeFilterItem> for WholeGenomeFilter {
    type Error = Error;

    fn try_from(source: WholeGenomeFilterItem) -> Result<Self, Self::Error> {
        use {WholeGenomeFilterKind as Kind, WholeGenomeFilterType as Type};

        let kind = match source.filter {
            Type::AssemblyLevel => {
                Kind::AssemblyLevel(from_value::<AssemblyLevel>(Value::String(source.value))?.into())
            }
            Type::GenomeRepresentation => {
                Kind::GenomeRepresentation(from_value::<GenomeRepresentation>(Value::String(source.value))?.into())
            }
            Type::ReleaseType => Kind::ReleaseType(from_value::<ReleaseType>(Value::String(source.value))?.into()),
        };

        Ok(match source.action {
            FilterAction::Include => WholeGenomeFilter::Include(kind),
            FilterAction::Exclude => WholeGenomeFilter::Exclude(kind),
        })
    }
}


pub fn convert_whole_genome_filters(items: Vec<WholeGenomeFilterItem>) -> Result<Vec<WholeGenomeFilter>, Error> {
    let mut filters = Vec::new();
    for item in items {
        filters.push(item.try_into()?);
    }
    Ok(filters)
}


/// An all purpose filter to apply to search queries.
#[derive(Clone, Debug, Serialize, Deserialize, InputObject)]
pub struct SearchFilterItem {
    filter: SearchFilterType,
    action: FilterAction,
    value: String,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum SearchFilterType {
    DataType,
}

/// Converts a graphql search filter into the search filter enum
impl TryFrom<SearchFilterItem> for SearchFilter {
    type Error = Error;

    fn try_from(source: SearchFilterItem) -> Result<Self, Self::Error> {
        use SearchFilterType as Type;

        let kind = match source.filter {
            Type::DataType => SearchFilter::DataType(from_value::<SearchDataType>(Value::String(source.value))?.into()),
        };

        Ok(kind)
    }
}


pub fn convert_search_filters(items: Vec<SearchFilterItem>) -> Result<Vec<SearchFilter>, Error> {
    let mut filters = Vec::new();
    for item in items {
        filters.push(item.try_into()?);
    }
    Ok(filters)
}
