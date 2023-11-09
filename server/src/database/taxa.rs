use arga_core::models::{TaxonomicRank, Classification};
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
    pub async fn find_by_taxon_rank(&self, taxon_rank: &TaxonRank) -> Result<Classification, Error> {
        use schema::classifications::dsl::*;

        let mut conn = self.pool.get().await?;

        let (taxon_rank, name) = match taxon_rank {
            TaxonRank::Domain(name) => (TaxonomicRank::Domain, name),
            TaxonRank::Kingdom(name) => (TaxonomicRank::Kingdom, name),
            TaxonRank::Phylum(name) => (TaxonomicRank::Phylum, name),
            TaxonRank::Class(name) => (TaxonomicRank::Class, name),
            TaxonRank::Order(name) => (TaxonomicRank::Order, name),
            TaxonRank::Family(name) => (TaxonomicRank::Family, name),
            TaxonRank::Genus(name) => (TaxonomicRank::Genus, name),
            TaxonRank::Species(name) => (TaxonomicRank::Species, name),
        };

        let taxon = classifications
            .filter(rank.eq(taxon_rank))
            .filter(canonical_name.eq(name))
            .first::<Classification>(&mut conn)
            .await?;

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
        use schema::taxa::dsl::*;
        use schema::classifications;
        use schema_gnl::classification_dag as dag;

        let mut conn = self.pool.get().await?;

        let species = taxa
            .inner_join(dag::table.on(parent_taxon_id.assume_not_null().eq(dag::taxon_id)))
            .inner_join(classifications::table.on(dag::id.eq(classifications::id)))
            .filter(status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .count()
            .into_boxed();

        let species = match rank {
            TaxonRank::Domain(name) => species
                .filter(classifications::rank.eq(TaxonomicRank::Domain))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Kingdom(name) => species
                .filter(classifications::rank.eq(TaxonomicRank::Kingdom))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Phylum(name) => species
                .filter(classifications::rank.eq(TaxonomicRank::Phylum))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Class(name) => species
                .filter(classifications::rank.eq(TaxonomicRank::Class))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Order(name) => species
                .filter(classifications::rank.eq(TaxonomicRank::Order))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Family(name) => species
                .filter(classifications::rank.eq(TaxonomicRank::Family))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Genus(name) => species
                .filter(classifications::rank.eq(TaxonomicRank::Genus))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Species(name) => species
                .filter(classifications::rank.eq(TaxonomicRank::Species))
                .filter(classifications::canonical_name.eq(name)),
        };

        let species = species
            .get_result::<i64>(&mut conn)
            .await?;

        let child = diesel::alias!(classifications as children);
        let children = classifications::table
            .inner_join(child.on(child.field(classifications::parent_id).eq(classifications::id)))
            .count()
            .into_boxed();

        let children = match rank {
            TaxonRank::Domain(name) => children
                .filter(classifications::rank.eq(TaxonomicRank::Domain))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Kingdom(name) => children
                .filter(classifications::rank.eq(TaxonomicRank::Kingdom))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Phylum(name) => children
                .filter(classifications::rank.eq(TaxonomicRank::Phylum))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Class(name) => children
                .filter(classifications::rank.eq(TaxonomicRank::Class))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Order(name) => children
                .filter(classifications::rank.eq(TaxonomicRank::Order))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Family(name) => children
                .filter(classifications::rank.eq(TaxonomicRank::Family))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Genus(name) => children
                .filter(classifications::rank.eq(TaxonomicRank::Genus))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Species(name) => children
                .filter(classifications::rank.eq(TaxonomicRank::Species))
                .filter(classifications::canonical_name.eq(name)),
        };

        let children = children
            .get_result::<i64>(&mut conn)
            .await?;

        Ok(TaxonSummary {
            children,
            species,
        })
    }


    pub async fn species_summary(&self, rank: &TaxonRank) -> Result<Vec<SpeciesSummary>, Error> {
        use schema::taxa;
        use schema_gnl::{name_data_summaries, classification_dag as dag};
        let mut conn = self.pool.get().await?;

        let summaries = taxa::table
            .inner_join(dag::table.on(dag::taxon_id.eq(taxa::parent_taxon_id.assume_not_null())))
            .inner_join(name_data_summaries::table.on(taxa::name_id.eq(name_data_summaries::name_id)))
            .filter(taxa::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .select((
                taxa::canonical_name,
                name_data_summaries::markers,
                name_data_summaries::genomes,
                name_data_summaries::specimens,
                name_data_summaries::other,
            ))
            .into_boxed();

        let summaries = match rank {
            TaxonRank::Domain(name) => summaries
                .filter(dag::rank.eq(TaxonomicRank::Domain))
                .filter(dag::canonical_name.eq(name)),
            TaxonRank::Kingdom(name) => summaries
                .filter(dag::rank.eq(TaxonomicRank::Kingdom))
                .filter(dag::canonical_name.eq(name)),
            TaxonRank::Phylum(name) => summaries
                .filter(dag::rank.eq(TaxonomicRank::Phylum))
                .filter(dag::canonical_name.eq(name)),
            TaxonRank::Class(name) => summaries
                .filter(dag::rank.eq(TaxonomicRank::Class))
                .filter(dag::canonical_name.eq(name)),
            TaxonRank::Order(name) => summaries
                .filter(dag::rank.eq(TaxonomicRank::Order))
                .filter(dag::canonical_name.eq(name)),
            TaxonRank::Family(name) => summaries
                .filter(dag::rank.eq(TaxonomicRank::Family))
                .filter(dag::canonical_name.eq(name)),
            TaxonRank::Genus(name) => summaries
                .filter(dag::rank.eq(TaxonomicRank::Genus))
                .filter(dag::canonical_name.eq(name)),
            TaxonRank::Species(name) => summaries
                .filter(dag::rank.eq(TaxonomicRank::Species))
                .filter(dag::canonical_name.eq(name)),
        };

        let summaries = summaries
            .load::<SpeciesSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }


    pub async fn data_summary(&self, rank: &TaxonRank) -> Result<Vec<DataSummary>, Error> {
        use diesel::dsl::sum;
        use schema::{taxa, classifications};
        use schema_gnl::{name_data_summaries, classification_dag as dag};
        let mut conn = self.pool.get().await?;

        let summaries = taxa::table
            .inner_join(dag::table.on(dag::taxon_id.eq(taxa::parent_taxon_id.assume_not_null())))
            .inner_join(classifications::table.on(dag::parent_id.eq(classifications::id)))
            .inner_join(name_data_summaries::table.on(taxa::name_id.eq(name_data_summaries::name_id)))
            .group_by(dag::canonical_name)
            .filter(taxa::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .select((
                dag::canonical_name.nullable(),
                sum(name_data_summaries::markers),
                sum(name_data_summaries::genomes),
                sum(name_data_summaries::specimens),
                sum(name_data_summaries::other),
            ))
            .into_boxed();

        let summaries = match rank {
            TaxonRank::Domain(name) => summaries
                .filter(classifications::rank.eq(TaxonomicRank::Domain))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Kingdom(name) => summaries
                .filter(classifications::rank.eq(TaxonomicRank::Kingdom))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Phylum(name) => summaries
                .filter(classifications::rank.eq(TaxonomicRank::Phylum))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Class(name) => summaries
                .filter(classifications::rank.eq(TaxonomicRank::Class))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Order(name) => summaries
                .filter(classifications::rank.eq(TaxonomicRank::Order))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Family(name) => summaries
                .filter(classifications::rank.eq(TaxonomicRank::Family))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Genus(name) => summaries
                .filter(classifications::rank.eq(TaxonomicRank::Genus))
                .filter(classifications::canonical_name.eq(name)),
            TaxonRank::Species(name) => summaries
                .filter(classifications::rank.eq(TaxonomicRank::Species))
                .filter(classifications::canonical_name.eq(name)),
        };

        let summaries = summaries
            .load::<DataSummary>(&mut conn)
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
        use schema::{taxa, classifications};
        use schema_gnl::{name_data_summaries, classification_dag as dag};
        let mut conn = self.pool.get().await?;

        let summaries = taxa::table
            .inner_join(dag::table.on(dag::taxon_id.eq(taxa::parent_taxon_id.assume_not_null())))
            .inner_join(classifications::table.on(dag::parent_id.eq(classifications::id)))
            .inner_join(name_data_summaries::table.on(taxa::name_id.eq(name_data_summaries::name_id)))
            .group_by(dag::canonical_name)
            .filter(taxa::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .filter(classifications::rank.eq(TaxonomicRank::Phylum))
            .filter(classifications::canonical_name.eq(phylum))
            .select((
                dag::canonical_name.nullable(),
                sum(name_data_summaries::markers),
                sum(name_data_summaries::genomes),
                sum(name_data_summaries::specimens),
                sum(name_data_summaries::other),
            ))
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
