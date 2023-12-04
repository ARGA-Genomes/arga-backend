use async_graphql::{Enum, InputObject, from_value, Value};
use serde::{Serialize, Deserialize};

use arga_core::search::SearchFilter;

use crate::http::Error;
use crate::database::extensions::filters::{Filter, FilterKind, Classification};
use crate::database::extensions::whole_genome_filters::{
    Filter as WholeGenomeFilter,
    FilterKind as WholeGenomeFilterKind,
};
use super::attributes::BushfireRecoveryTrait;
use super::species::DataType;
use super::search::SearchDataType;
use super::taxonomy::TaxonomicVernacularGroup;
use super::whole_genomes::{AssemblyLevel, GenomeRepresentation, ReleaseType};


#[derive(Clone, Debug, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum FilterType {
    Kingdom,
    Phylum,
    Class,
    Order,
    Family,
    Tribe,
    Genus,

    VernacularGroup,
    HasData,
    Ecology,
    Ibra,
    Imcra,
    State,
    DrainageBasin,

    BushfireRecovery,
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
            FilterType::Kingdom => FilterKind::Classification(Classification::Kingdom(source.value)),
            FilterType::Phylum => FilterKind::Classification(Classification::Phylum(source.value)),
            FilterType::Class => FilterKind::Classification(Classification::Class(source.value)),
            FilterType::Order => FilterKind::Classification(Classification::Order(source.value)),
            FilterType::Family => FilterKind::Classification(Classification::Family(source.value)),
            FilterType::Tribe => FilterKind::Classification(Classification::Tribe(source.value)),
            FilterType::Genus => FilterKind::Classification(Classification::Genus(source.value)),

            FilterType::VernacularGroup => FilterKind::VernacularGroup(
                from_value::<TaxonomicVernacularGroup>(Value::String(source.value))?.into()
            ),

            FilterType::HasData => FilterKind::HasData(
                from_value::<DataType>(Value::String(source.value))?.into()
            ),

            FilterType::Ecology => FilterKind::Ecology(source.value),
            FilterType::Ibra => FilterKind::Ibra(source.value),
            FilterType::Imcra => FilterKind::Imcra(source.value),
            FilterType::State => FilterKind::State(source.value),
            FilterType::DrainageBasin => FilterKind::DrainageBasin(source.value),

            FilterType::BushfireRecovery => FilterKind::BushfireRecovery(
                from_value::<BushfireRecoveryTrait>(Value::String(source.value))?.into()
            ),
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
        use WholeGenomeFilterType as Type;
        use WholeGenomeFilterKind as Kind;

        let kind = match source.filter {
            Type::AssemblyLevel => Kind::AssemblyLevel(
                from_value::<AssemblyLevel>(Value::String(source.value))?.into()
            ),
            Type::GenomeRepresentation => Kind::GenomeRepresentation(
                from_value::<GenomeRepresentation>(Value::String(source.value))?.into()
            ),
            Type::ReleaseType => Kind::ReleaseType(
                from_value::<ReleaseType>(Value::String(source.value))?.into()
            ),
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
            Type::DataType => SearchFilter::DataType(
                from_value::<SearchDataType>(Value::String(source.value))?.into()
            ),
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
