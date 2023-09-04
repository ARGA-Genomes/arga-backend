use async_graphql::{Enum, InputObject};
use serde::{Serialize, Deserialize};

use crate::database::extensions::filters::{Filter, FilterKind};


#[derive(Clone, Debug, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum FilterType {
    Kingdom,
    Phylum,
    Class,
    Order,
    Family,
    Tribe,
    Genus,
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
impl From<FilterItem> for Filter {
    fn from(source: FilterItem) -> Self {
        let kind = match source.filter {
            FilterType::Kingdom => FilterKind::Kingdom(source.value),
            FilterType::Phylum => FilterKind::Phylum(source.value),
            FilterType::Class => FilterKind::Class(source.value),
            FilterType::Order => FilterKind::Order(source.value),
            FilterType::Family => FilterKind::Family(source.value),
            FilterType::Tribe => FilterKind::Tribe(source.value),
            FilterType::Genus => FilterKind::Genus(source.value),
        };

        match source.action {
            FilterAction::Include => Filter::Include(kind),
            FilterAction::Exclude => Filter::Exclude(kind),
        }
    }
}


pub fn convert_filters(items: Vec<FilterItem>) -> Vec<Filter> {
    items.into_iter().map(|f| f.into()).collect()
}
