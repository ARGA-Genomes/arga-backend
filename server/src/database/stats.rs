use arga_core::models::Dataset;
use async_graphql::SimpleObject;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Serialize, Deserialize};

use super::models::TaxonomicStatus;
use super::extensions::sum_if;
use super::{schema, Error, PgPool};


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenusStats {
    /// The total amount of accepted species in the genus
    pub total_valid_species: usize,
    /// The total amount of species in the genus
    pub total_species: usize,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenusBreakdown {
    pub species: Vec<BreakdownItem>,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyStats {
    /// The total amount of genera in the family
    pub total_genera: usize,
    pub total_genera_with_data: usize,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyBreakdown {
    pub genera: Vec<BreakdownItem>,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassStats {
    /// The total amount of orders in the class
    pub total_orders: usize,
    pub total_orders_with_data: usize,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassBreakdown {
    pub orders: Vec<BreakdownItem>,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderStats {
    /// The total amount of families in the order
    pub total_families: usize,
    pub total_families_with_data: usize,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBreakdown {
    pub families: Vec<BreakdownItem>,
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetStats {
    /// The total amount of species in the order
    pub total_species: usize,
    pub total_species_with_data: usize,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetBreakdown {
    pub species: Vec<BreakdownItem>,
}


#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct BreakdownItem {
    pub name: Option<String>,
    pub total: i64,
}


#[derive(Clone)]
pub struct StatsProvider {
    pub pool: PgPool,
}

impl StatsProvider {
    /// Gets stats for a specific genus.
    pub async fn genus(&self, name: &str) -> Result<GenusStats, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let (valid, total): (i64, i64) = taxa
            .select((
                sum_if(status.eq(TaxonomicStatus::Accepted)),
                diesel::dsl::count_star()
            ))
            .filter(genus.eq(name))
            .group_by(genus)
            .get_result(&mut conn)
            .await?;

        Ok(GenusStats {
            // this can never be negative due to the count
            total_valid_species: valid as usize,
            total_species: total as usize
        })
    }

    pub async fn genus_breakdown(&self, genus_value: &str) -> Result<GenusBreakdown, Error> {
        use schema::{names, taxa, assemblies};
        use diesel::dsl::count_star;

        let mut conn = self.pool.get().await?;

        let species = assemblies::table
            .inner_join(names::table)
            .inner_join(taxa::table.on(names::id.eq(taxa::name_id)))
            .filter(taxa::genus.eq(genus_value))
            .filter(taxa::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .group_by(names::canonical_name)
            .select((names::canonical_name.nullable(), count_star()))
            .load::<BreakdownItem>(&mut conn)
            .await?;

        Ok(GenusBreakdown { species })
    }


    pub async fn family(&self, family_value: &str) -> Result<FamilyStats, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let total_genera: i64 = taxa
            .filter(family.eq(family_value))
            .filter(status.eq(TaxonomicStatus::Accepted))
            .group_by(genus)
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(FamilyStats {
            // this can never be negative due to the count
            total_genera: total_genera as usize,
            total_genera_with_data: 0,
        })
    }

    pub async fn family_breakdown(&self, name: &str) -> Result<FamilyBreakdown, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let genera = taxa
            .filter(genus.eq(name))
            .filter(status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .group_by(genus)
            .select((genus, diesel::dsl::count_star()))
            .load::<BreakdownItem>(&mut conn)
            .await?;

        Ok(FamilyBreakdown {
            genera,
        })
    }

    pub async fn order(&self, name: &str) -> Result<OrderStats, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = taxa
            .filter(order.eq(name))
            .filter(status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .group_by(family)
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(OrderStats {
            // this can never be negative due to the count
            total_families: total as usize,
            total_families_with_data: 0,
        })
    }

    pub async fn order_breakdown(&self, name: &str) -> Result<OrderBreakdown, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let families = taxa
            .filter(order.eq(name))
            .filter(status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .group_by(family)
            .select((family, diesel::dsl::count_star()))
            .load::<BreakdownItem>(&mut conn)
            .await?;

        Ok(OrderBreakdown {
            families,
        })
    }

    pub async fn class(&self, name: &str) -> Result<ClassStats, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = taxa
            .filter(class.eq(name))
            .filter(status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .group_by(order)
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(ClassStats {
            // this can never be negative due to the count
            total_orders: total as usize,
            total_orders_with_data: 0,
        })
    }

    pub async fn class_breakdown(&self, name: &str) -> Result<ClassBreakdown, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let orders = taxa
            .filter(class.eq(name))
            .filter(status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .group_by(order)
            .select((order, diesel::dsl::count_star()))
            .load::<BreakdownItem>(&mut conn)
            .await?;

        Ok(ClassBreakdown {
            orders,
        })
    }

    pub async fn dataset(&self, name: &str) -> Result<DatasetStats, Error> {
        use schema::{datasets, taxa, indigenous_knowledge as iek};
        let mut conn = self.pool.get().await?;

        let dataset = datasets::table
            .filter(datasets::name.eq(&name))
            .get_result::<Dataset>(&mut conn)
            .await?;

        let total: i64 = taxa::table
            .left_join(iek::table.on(taxa::name_id.eq(iek::name_id)))
            .filter(iek::dataset_id.eq(dataset.id))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(DatasetStats {
            // this can never be negative due to the count
            total_species: total as usize,
            total_species_with_data: 0,
        })
    }

    pub async fn dataset_breakdown(&self, _name: &str) -> Result<DatasetBreakdown, Error> {
        let species = vec![];

        Ok(DatasetBreakdown {
            species,
        })
    }
}
