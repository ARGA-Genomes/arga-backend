use diesel::prelude::*;
use diesel::sql_types::{Text, Array, Nullable};
use diesel_async::RunQueryDsl;

use crate::database::extensions::filters::{with_filters, Filter};

use super::extensions::Paginate;
use super::extensions::filters::filter_taxa;
use super::{schema, schema_gnl, PgPool, PageResult, Error};
use super::models::{Taxon, FilteredTaxon, TaxonomicStatus};


sql_function!(fn unnest(x: Nullable<Array<Text>>) -> Text);


pub enum TaxonRank {
    Kingdom(String),
    Phylum(String),
    Class(String),
    Order(String),
    Family(String),
    Genus(String),
    Species(String),
}

#[derive(Debug, Queryable)]
pub struct DataSummary {
    pub rank: Option<String>,
    pub markers: Option<i64>,
    pub genomes: Option<i64>,
    pub specimens: Option<i64>,
    pub other: Option<i64>,
}


#[derive(Clone)]
pub struct TaxaProvider {
    pub pool: PgPool,
}

impl TaxaProvider {
    pub async fn find_by_taxon_rank(&self, rank: &TaxonRank) -> Result<Taxon, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let taxon = match rank {
            TaxonRank::Kingdom(name) => taxa.filter(kingdom.eq(name)).get_result::<Taxon>(&mut conn),
            TaxonRank::Phylum(name) => taxa.filter(phylum.eq(name)).get_result::<Taxon>(&mut conn),
            TaxonRank::Class(name) => taxa.filter(class.eq(name)).get_result::<Taxon>(&mut conn),
            TaxonRank::Order(name) => taxa.filter(order.eq(name)).get_result::<Taxon>(&mut conn),
            TaxonRank::Family(name) => taxa.filter(family.eq(name)).get_result::<Taxon>(&mut conn),
            TaxonRank::Genus(name) => taxa.filter(genus.eq(name)).get_result::<Taxon>(&mut conn),
            TaxonRank::Species(name) => taxa.filter(canonical_name.eq(name)).get_result::<Taxon>(&mut conn),
        }.await?;

        Ok(taxon)
    }

    pub async fn species(&self, filters: &Vec<Filter>, page: i64, per_page: i64) -> PageResult<FilteredTaxon> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let species = taxa_filter::table
            .select(taxa_filter::all_columns)
            .filter(taxa_filter::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .filter(with_filters(&filters).unwrap())
            .order_by(taxa_filter::scientific_name)
            .paginate(page)
            .per_page(per_page)
            .load::<(FilteredTaxon, i64)>(&mut conn)
            .await?;

        Ok(species.into())
    }

    pub async fn ecology_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let mut options = filter_taxa(filters)
            .select(unnest(taxa_filter::ecology))
            .distinct()
            .load::<String>(&mut conn)
            .await?;

        options.sort();
        Ok(options)
    }

    pub async fn ibra_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let mut options = filter_taxa(filters)
            .select(unnest(taxa_filter::ibra))
            .distinct()
            .load::<String>(&mut conn)
            .await?;

        options.sort();
        Ok(options)
    }

    pub async fn imcra_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let mut options = filter_taxa(filters)
            .select(unnest(taxa_filter::ibra))
            .distinct()
            .load::<String>(&mut conn)
            .await?;

        options.sort();
        Ok(options)
    }

    pub async fn state_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let mut options = filter_taxa(filters)
            .select(unnest(taxa_filter::state))
            .distinct()
            .load::<String>(&mut conn)
            .await?;

        options.sort();
        Ok(options)
    }

    pub async fn drainage_basin_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let mut options = filter_taxa(filters)
            .select(unnest(taxa_filter::drainage_basin))
            .distinct()
            .load::<String>(&mut conn)
            .await?;

        options.sort();
        Ok(options)
    }


    pub async fn kingdom_summary(&self, kingdom: &str) -> Result<Vec<DataSummary>, Error> {
        use diesel::dsl::sum;
        use schema_gnl::{name_data_summaries, taxa_filter};
        let mut conn = self.pool.get().await?;

        let summaries = name_data_summaries::table
            .inner_join(taxa_filter::table.on(taxa_filter::name_id.eq(name_data_summaries::name_id)))
            .group_by(taxa_filter::kingdom)
            .select((
                taxa_filter::kingdom,
                sum(name_data_summaries::markers),
                sum(name_data_summaries::genomes),
                sum(name_data_summaries::specimens),
                sum(name_data_summaries::other),
            ))
            .filter(taxa_filter::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .filter(taxa_filter::kingdom.eq(kingdom))
            // .filter(with_filters(&filters).unwrap())
            .load::<DataSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn phylum_summary(&self, phylum: &str) -> Result<Vec<DataSummary>, Error> {
        use diesel::dsl::sum;
        use schema_gnl::{name_data_summaries, taxa_filter};
        let mut conn = self.pool.get().await?;

        let summaries = name_data_summaries::table
            .inner_join(taxa_filter::table.on(taxa_filter::name_id.eq(name_data_summaries::name_id)))
            .group_by(taxa_filter::class)
            .select((
                taxa_filter::class,
                sum(name_data_summaries::markers),
                sum(name_data_summaries::genomes),
                sum(name_data_summaries::specimens),
                sum(name_data_summaries::other),
            ))
            .filter(taxa_filter::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .filter(taxa_filter::phylum.eq(phylum))
            .load::<DataSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn class_summary(&self, class: &str) -> Result<Vec<DataSummary>, Error> {
        use diesel::dsl::sum;
        use schema_gnl::{name_data_summaries, taxa_filter};
        let mut conn = self.pool.get().await?;

        let summaries = name_data_summaries::table
            .inner_join(taxa_filter::table.on(taxa_filter::name_id.eq(name_data_summaries::name_id)))
            .group_by(taxa_filter::order)
            .select((
                taxa_filter::order,
                sum(name_data_summaries::markers),
                sum(name_data_summaries::genomes),
                sum(name_data_summaries::specimens),
                sum(name_data_summaries::other),
            ))
            .filter(taxa_filter::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .filter(taxa_filter::class.eq(class))
            .load::<DataSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn order_summary(&self, order: &str) -> Result<Vec<DataSummary>, Error> {
        use diesel::dsl::sum;
        use schema_gnl::{name_data_summaries, taxa_filter};
        let mut conn = self.pool.get().await?;

        let summaries = name_data_summaries::table
            .inner_join(taxa_filter::table.on(taxa_filter::name_id.eq(name_data_summaries::name_id)))
            .group_by(taxa_filter::family)
            .select((
                taxa_filter::family,
                sum(name_data_summaries::markers),
                sum(name_data_summaries::genomes),
                sum(name_data_summaries::specimens),
                sum(name_data_summaries::other),
            ))
            .filter(taxa_filter::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .filter(taxa_filter::order.eq(order))
            .load::<DataSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn family_summary(&self, family: &str) -> Result<Vec<DataSummary>, Error> {
        use diesel::dsl::sum;
        use schema_gnl::{name_data_summaries, taxa_filter};
        let mut conn = self.pool.get().await?;

        let summaries = name_data_summaries::table
            .inner_join(taxa_filter::table.on(taxa_filter::name_id.eq(name_data_summaries::name_id)))
            .group_by(taxa_filter::genus)
            .select((
                taxa_filter::genus,
                sum(name_data_summaries::markers),
                sum(name_data_summaries::genomes),
                sum(name_data_summaries::specimens),
                sum(name_data_summaries::other),
            ))
            .filter(taxa_filter::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .filter(taxa_filter::family.eq(family))
            .load::<DataSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }
}
