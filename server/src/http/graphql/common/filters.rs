use async_graphql::{Enum, InputObject, from_value, Value};
use serde::{Serialize, Deserialize};

use crate::http::Error;
use crate::database::extensions::filters::{Filter, FilterKind};
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
            FilterType::Kingdom => FilterKind::Kingdom(source.value),
            FilterType::Phylum => FilterKind::Phylum(source.value),
            FilterType::Class => FilterKind::Class(source.value),
            FilterType::Order => FilterKind::Order(source.value),
            FilterType::Family => FilterKind::Family(source.value),
            FilterType::Tribe => FilterKind::Tribe(source.value),
            FilterType::Genus => FilterKind::Genus(source.value),

            FilterType::VernacularGroup => FilterKind::VernacularGroup(
                from_value::<TaxonomicVernacularGroup>(Value::String(source.value))?.into()
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
