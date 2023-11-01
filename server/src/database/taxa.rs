use diesel::prelude::*;
use diesel::sql_types::{Text, Array, Nullable};
use diesel_async::RunQueryDsl;

use crate::database::extensions::filters::{with_filters, Filter};

use super::extensions::Paginate;
use super::extensions::filters::filter_taxa;
use super::{schema, schema_gnl, PgPool, PageResult, Error};
use super::models::{Taxon, FilteredTaxon, TaxonomicStatus};


sql_function!(fn unnest(x: Nullable<Array<Text>>) -> Text);


#[derive(Clone, Debug)]
pub enum TaxonRank {
    Domain(String),
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

#[derive(Debug, Queryable)]
pub struct SpeciesSummary {
    pub name: String,
    pub markers: Option<i32>,
    pub genomes: Option<i32>,
    pub specimens: Option<i32>,
    pub other: Option<i32>,
}

#[derive(Debug, Queryable)]
pub struct TaxonSummary {
    pub children: i64,
    pub species: i64,
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
            TaxonRank::Domain(name) => taxa.get_result::<Taxon>(&mut conn),
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


    pub async fn taxon_summary(&self, rank: &TaxonRank) -> Result<TaxonSummary, Error> {
        use diesel::dsl::{count, count_star};
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let species = taxa
            .filter(status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .count()
            .into_boxed();

        let species = match rank {
            TaxonRank::Domain(_name) => species,
            TaxonRank::Kingdom(name) => species.filter(kingdom.eq(name)),
            TaxonRank::Phylum(name) => species.filter(phylum.eq(name)),
            TaxonRank::Class(name) => species.filter(class.eq(name)),
            TaxonRank::Order(name) => species.filter(order.eq(name)),
            TaxonRank::Family(name) => species.filter(family.eq(name)),
            TaxonRank::Genus(name) => species.filter(genus.eq(name)),
            TaxonRank::Species(name) => species.filter(canonical_name.eq(name)),
        };

        let species = species
            .get_result::<i64>(&mut conn)
            .await?;

        let children = match rank {
            TaxonRank::Domain(_name) => taxa.select(count_star()).group_by(kingdom).load::<i64>(&mut conn).await?,
            TaxonRank::Kingdom(name) => taxa.select(count_star()).filter(kingdom.eq(name)).group_by(phylum).load::<i64>(&mut conn).await?,
            TaxonRank::Phylum(name) => taxa.select(count_star()).filter(phylum.eq(name)).group_by(class).load::<i64>(&mut conn).await?,
            TaxonRank::Class(name) => taxa.select(count_star()).filter(class.eq(name)).group_by(order).load::<i64>(&mut conn).await?,
            TaxonRank::Order(name) => taxa.select(count(family)).filter(order.eq(name)).group_by(family).load::<i64>(&mut conn).await?,
            TaxonRank::Family(name) => taxa.select(count_star()).filter(family.eq(name)).group_by(genus).load::<i64>(&mut conn).await?,
            TaxonRank::Genus(name) => taxa.select(count_star()).filter(genus.eq(name)).group_by(genus).load::<i64>(&mut conn).await?,
            TaxonRank::Species(name) => taxa.select(count_star()).filter(canonical_name.eq(name)).group_by(canonical_name).load::<i64>(&mut conn).await?,
        };

        Ok(TaxonSummary {
            children: children.len() as i64,
            species,
        })
    }


    pub async fn species_summary(&self, rank: &TaxonRank) -> Result<Vec<SpeciesSummary>, Error> {
        use schema_gnl::{name_data_summaries, taxa_filter};
        let mut conn = self.pool.get().await?;

        let summaries = name_data_summaries::table
            .inner_join(taxa_filter::table.on(taxa_filter::name_id.eq(name_data_summaries::name_id)))
            .select((
                taxa_filter::canonical_name,
                name_data_summaries::markers,
                name_data_summaries::genomes,
                name_data_summaries::specimens,
                name_data_summaries::other,
            ))
            .filter(taxa_filter::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .into_boxed();

        let summaries = match rank {
            TaxonRank::Domain(_name) => summaries,
            TaxonRank::Kingdom(name) => summaries.filter(taxa_filter::kingdom.eq(name)),
            TaxonRank::Phylum(name) => summaries.filter(taxa_filter::phylum.eq(name)),
            TaxonRank::Class(name) => summaries.filter(taxa_filter::class.eq(name)),
            TaxonRank::Order(name) => summaries.filter(taxa_filter::order.eq(name)),
            TaxonRank::Family(name) => summaries.filter(taxa_filter::family.eq(name)),
            TaxonRank::Genus(name) => summaries.filter(taxa_filter::genus.eq(name)),
            TaxonRank::Species(name) => summaries.filter(taxa_filter::canonical_name.eq(name)),
        };

        let summaries = summaries
            .load::<SpeciesSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }


    pub async fn domain_summary(&self, _domain: &str) -> Result<Vec<DataSummary>, Error> {
        use diesel::dsl::sum;
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let summaries = taxa_filter::table
            .group_by(taxa_filter::kingdom)
            .select((
                taxa_filter::kingdom,
                sum(taxa_filter::markers),
                sum(taxa_filter::genomes),
                sum(taxa_filter::specimens),
                sum(taxa_filter::other),
            ))
            .filter(taxa_filter::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .load::<DataSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn kingdom_summary(&self, kingdom: &str) -> Result<Vec<DataSummary>, Error> {
        use diesel::dsl::sum;
        use schema_gnl::{name_data_summaries, taxa_filter};
        let mut conn = self.pool.get().await?;

        let summaries = name_data_summaries::table
            .inner_join(taxa_filter::table.on(taxa_filter::name_id.eq(name_data_summaries::name_id)))
            .group_by(taxa_filter::phylum)
            .select((
                taxa_filter::phylum,
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
