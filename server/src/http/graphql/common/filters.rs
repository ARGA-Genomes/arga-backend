use async_graphql::{Enum, InputObject, Value, from_value};
use serde::{Deserialize, Serialize};

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
    Attribute,
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
    value: serde_json::Value,
}

fn filter_value_string(value: serde_json::Value) -> String {
    if value.is_string() {
        return value.as_str().unwrap().to_string();
    }

    String::from("INVALID-VALUE-TYPE")
}

/// Converts a graphql filter into the common filter enum
impl TryFrom<FilterItem> for Filter {
    type Error = Error;

    fn try_from(source: FilterItem) -> Result<Self, Self::Error> {
        let kind = match source.filter {
            FilterType::VernacularGroup => FilterKind::VernacularGroup(
                from_value::<TaxonomicVernacularGroup>(Value::String(filter_value_string(source.value).to_string()))?
                    .into(),
            ),

            FilterType::HasData => {
                FilterKind::HasData(from_value::<DataType>(Value::String(filter_value_string(source.value)))?.into())
            }
            FilterType::Attribute => FilterKind::Attribute(source.value),

            // FilterType::Ecology => FilterKind::Ecology(filter_value_string(source.value)),
            // FilterType::Ibra => FilterKind::Ibra(filter_value_string(source.value)),
            // FilterType::Imcra => FilterKind::Imcra(filter_value_string(source.value)),
            // FilterType::State => FilterKind::State(filter_value_string(source.value)),
            // FilterType::DrainageBasin => FilterKind::DrainageBasin(filter_value_string(source.value)),

            // FilterType::BushfireRecovery => FilterKind::BushfireRecovery(
            //     from_value::<BushfireRecoveryTrait>(Value::String(filter_value_string(source.value)))?.into()
            // ),
            FilterType::Domain => FilterKind::Classification(Classification::Domain(filter_value_string(source.value))),
            FilterType::Superkingdom => {
                FilterKind::Classification(Classification::Superkingdom(filter_value_string(source.value)))
            }
            FilterType::Kingdom => {
                FilterKind::Classification(Classification::Kingdom(filter_value_string(source.value)))
            }
            FilterType::Subkingdom => {
                FilterKind::Classification(Classification::Subkingdom(filter_value_string(source.value)))
            }
            FilterType::Phylum => FilterKind::Classification(Classification::Phylum(filter_value_string(source.value))),
            FilterType::Subphylum => {
                FilterKind::Classification(Classification::Subphylum(filter_value_string(source.value)))
            }
            FilterType::Superclass => {
                FilterKind::Classification(Classification::Superclass(filter_value_string(source.value)))
            }
            FilterType::Class => FilterKind::Classification(Classification::Class(filter_value_string(source.value))),
            FilterType::Subclass => {
                FilterKind::Classification(Classification::Subclass(filter_value_string(source.value)))
            }
            FilterType::Superorder => {
                FilterKind::Classification(Classification::Superorder(filter_value_string(source.value)))
            }
            FilterType::Order => FilterKind::Classification(Classification::Order(filter_value_string(source.value))),
            FilterType::Suborder => {
                FilterKind::Classification(Classification::Suborder(filter_value_string(source.value)))
            }
            FilterType::Hyporder => {
                FilterKind::Classification(Classification::Hyporder(filter_value_string(source.value)))
            }
            FilterType::Superfamily => {
                FilterKind::Classification(Classification::Superfamily(filter_value_string(source.value)))
            }
            FilterType::Family => FilterKind::Classification(Classification::Family(filter_value_string(source.value))),
            FilterType::Subfamily => {
                FilterKind::Classification(Classification::Subfamily(filter_value_string(source.value)))
            }
            FilterType::Supertribe => {
                FilterKind::Classification(Classification::Supertribe(filter_value_string(source.value)))
            }
            FilterType::Tribe => FilterKind::Classification(Classification::Tribe(filter_value_string(source.value))),
            FilterType::Subtribe => {
                FilterKind::Classification(Classification::Subtribe(filter_value_string(source.value)))
            }
            FilterType::Genus => FilterKind::Classification(Classification::Genus(filter_value_string(source.value))),
            FilterType::Subgenus => {
                FilterKind::Classification(Classification::Subgenus(filter_value_string(source.value)))
            }
            FilterType::Cohort => FilterKind::Classification(Classification::Cohort(filter_value_string(source.value))),
            FilterType::Subcohort => {
                FilterKind::Classification(Classification::Subcohort(filter_value_string(source.value)))
            }
            FilterType::Division => {
                FilterKind::Classification(Classification::Division(filter_value_string(source.value)))
            }
            FilterType::Subdivision => {
                FilterKind::Classification(Classification::Subdivision(filter_value_string(source.value)))
            }
            FilterType::Section => {
                FilterKind::Classification(Classification::Section(filter_value_string(source.value)))
            }
            FilterType::Regnum => FilterKind::Classification(Classification::Regnum(filter_value_string(source.value))),
            FilterType::Familia => {
                FilterKind::Classification(Classification::Familia(filter_value_string(source.value)))
            }
            FilterType::Classis => {
                FilterKind::Classification(Classification::Classis(filter_value_string(source.value)))
            }
            FilterType::Ordo => FilterKind::Classification(Classification::Ordo(filter_value_string(source.value))),
            FilterType::Forma => FilterKind::Classification(Classification::Forma(filter_value_string(source.value))),
            FilterType::Subclassis => {
                FilterKind::Classification(Classification::Subclassis(filter_value_string(source.value)))
            }
            FilterType::Superordo => {
                FilterKind::Classification(Classification::Superordo(filter_value_string(source.value)))
            }
            FilterType::Sectio => FilterKind::Classification(Classification::Sectio(filter_value_string(source.value))),
            FilterType::Series => FilterKind::Classification(Classification::Series(filter_value_string(source.value))),
            FilterType::Subfamilia => {
                FilterKind::Classification(Classification::Subfamilia(filter_value_string(source.value)))
            }
            FilterType::Subordo => {
                FilterKind::Classification(Classification::Subordo(filter_value_string(source.value)))
            }
            FilterType::Regio => FilterKind::Classification(Classification::Regio(filter_value_string(source.value))),
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
