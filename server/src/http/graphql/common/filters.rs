use async_graphql::{Enum, InputObject, from_value, Value};
use serde::{Serialize, Deserialize};

use crate::http::Error;
use crate::database::extensions::filters::{Filter, FilterKind, Classification};
use super::attributes::BushfireRecoveryTrait;
use super::taxonomy::TaxonomicVernacularGroup;


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

/// An all purpose filter to apply the query.
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
