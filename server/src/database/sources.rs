use arga_core::models::Species;
use bigdecimal::{BigDecimal, Zero};
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::extensions::Paginate;
use super::extensions::date_utils::DateParser;
use super::models::{Dataset, Source};
use super::{PageResult, PgPool, schema, schema_gnl};
use crate::database::Error;
use crate::database::extensions::filters::{Filter, with_filters};
use crate::database::extensions::species_filters::{SortDirection, SpeciesSort, with_sorting};
use crate::database::extensions::sum_if;
use crate::database::taxa::RankSummary;

pub const ALA_DATASET_ID: &str = "ARGA:TL:0001013";

#[derive(Clone)]
pub struct SourceProvider {
    pub pool: PgPool,
}

#[derive(Clone)]
pub struct GenomeRelease {
    pub scientific_name: String,
    pub canonical_name: String,
    pub release_date: Option<NaiveDate>,
}


impl SourceProvider {
    pub async fn all_records(&self) -> Result<Vec<Source>, Error> {
        use schema::sources::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = sources.order_by(name).load::<Source>(&mut conn).await?;

        Ok(records)
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<Source, Error> {
        use schema::sources;
        let mut conn = self.pool.get().await?;

        let source = sources::table
            .filter(sources::id.eq(id))
            .get_result::<Source>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = source {
            return Err(Error::NotFound(id.to_string()));
        }

        Ok(source?)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Source, Error> {
        use schema::sources;
        let mut conn = self.pool.get().await?;

        let source = sources::table
            .filter(sources::name.eq(name))
            .get_result::<Source>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = source {
            return Err(Error::NotFound(name.to_string()));
        }

        Ok(source?)
    }

    pub async fn datasets(&self, source: &Source) -> Result<Vec<Dataset>, Error> {
        use schema::datasets;
        let mut conn = self.pool.get().await?;

        let records = datasets::table
            .filter(datasets::source_id.eq(source.id))
            .order_by(datasets::name)
            .load::<Dataset>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn species(
        &self,
        source: &Source,
        filters: &Vec<Filter>,
        page: i64,
        page_size: i64,
        sort: SpeciesSort,
        direction: SortDirection,
    ) -> PageResult<Species> {
        use schema::{datasets, name_attributes as attrs, taxon_names};
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let query = match with_filters(&filters) {
            Some(predicates) => species::table.filter(predicates).into_boxed(),
            None => species::table.into_boxed(),
        };

        let taxa_datasets = diesel::alias!(datasets as taxa_datasets);

        let records = with_sorting(query, sort, direction)
            .inner_join(taxon_names::table.on(species::id.eq(taxon_names::taxon_id)))
            .inner_join(attrs::table.on(attrs::name_id.eq(taxon_names::name_id)))
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .inner_join(taxa_datasets.on(taxa_datasets.field(datasets::id).eq(species::dataset_id)))
            .select(species::all_columns)
            .distinct()
            .filter(datasets::source_id.eq(source.id))
            .filter(taxa_datasets.field(datasets::global_id).eq(ALA_DATASET_ID))
            .paginate(page)
            .per_page(page_size)
            .load::<(Species, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    // the top 10 species that have the most genomic data for this source
    pub async fn species_genomic_data_summary(
        &self,
        source: &Source,
        filters: &Option<Vec<Filter>>,
    ) -> Result<Vec<super::taxa::Summary>, Error> {
        use schema::{datasets, name_attributes as attrs, taxa, taxon_names};
        use schema_gnl::{species, taxa_tree_stats as stats};
        let mut conn = self.pool.get().await?;

        // Use the same join pattern as the species() method to find species for this source
        let taxa_datasets = diesel::alias!(datasets as taxa_datasets);

        let query = match filters.as_ref().and_then(|f| with_filters(f)) {
            Some(predicates) => species::table.filter(predicates).into_boxed(),
            None => species::table.into_boxed(),
        };

        let source_species: Vec<uuid::Uuid> = query
            .inner_join(taxon_names::table.on(species::id.eq(taxon_names::taxon_id)))
            .inner_join(attrs::table.on(attrs::name_id.eq(taxon_names::name_id)))
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .inner_join(taxa_datasets.on(taxa_datasets.field(datasets::id).eq(species::dataset_id)))
            .select(species::id)
            .distinct()
            .filter(datasets::source_id.eq(source.id))
            .filter(taxa_datasets.field(datasets::global_id).eq(ALA_DATASET_ID))
            .load::<uuid::Uuid>(&mut conn)
            .await?;

        if source_species.is_empty() {
            return Ok(vec![]);
        }

        // Then get stats for those species from the taxa tree stats (no cross-schema join needed)
        let summaries = stats::table
            .inner_join(taxa::table.on(taxa::id.eq(stats::id)))
            .select((
                taxa::scientific_name,
                taxa::canonical_name,
                stats::genomes,
                stats::loci,
                stats::specimens,
                stats::other,
                stats::total_genomic,
            ))
            .filter(stats::id.eq_any(source_species))
            .filter(taxa::rank.eq(arga_core::models::TaxonomicRank::Species))
            .distinct()
            .order(stats::total_genomic.desc())
            .limit(10)
            .load::<super::taxa::Summary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    // the top 10 species that have the most genomes for this source
    pub async fn species_genomes_summary(
        &self,
        source: &Source,
        filters: &Option<Vec<Filter>>,
    ) -> Result<Vec<super::taxa::Summary>, Error> {
        use schema::{datasets, name_attributes as attrs, taxa, taxon_names};
        use schema_gnl::{species, taxa_tree_stats as stats};
        let mut conn = self.pool.get().await?;

        // Use the same join pattern as the species() method to find species for this source
        let taxa_datasets = diesel::alias!(datasets as taxa_datasets);

        let query = match filters.as_ref().and_then(|f| with_filters(f)) {
            Some(predicates) => species::table.filter(predicates).into_boxed(),
            None => species::table.into_boxed(),
        };

        let source_species: Vec<uuid::Uuid> = query
            .inner_join(taxon_names::table.on(species::id.eq(taxon_names::taxon_id)))
            .inner_join(attrs::table.on(attrs::name_id.eq(taxon_names::name_id)))
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .inner_join(taxa_datasets.on(taxa_datasets.field(datasets::id).eq(species::dataset_id)))
            .select(species::id)
            .distinct()
            .filter(datasets::source_id.eq(source.id))
            .filter(taxa_datasets.field(datasets::global_id).eq(ALA_DATASET_ID))
            .load::<uuid::Uuid>(&mut conn)
            .await?;

        if source_species.is_empty() {
            return Ok(vec![]);
        }

        // Then get stats for those species from the taxa tree stats (no cross-schema join needed)
        let summaries = stats::table
            .inner_join(taxa::table.on(taxa::id.eq(stats::id)))
            .select((
                taxa::scientific_name,
                taxa::canonical_name,
                stats::genomes,
                stats::loci,
                stats::specimens,
                stats::other,
                stats::total_genomic,
            ))
            .filter(stats::id.eq_any(source_species))
            .filter(taxa::rank.eq(arga_core::models::TaxonomicRank::Species))
            .distinct()
            .order(stats::genomes.desc())
            .limit(10)
            .load::<super::taxa::Summary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    // the top 10 species that have the most genomes for this source
    pub async fn species_loci_summary(
        &self,
        source: &Source,
        filters: &Option<Vec<Filter>>,
    ) -> Result<Vec<super::taxa::Summary>, Error> {
        use schema::{datasets, name_attributes as attrs, taxa, taxon_names};
        use schema_gnl::{species, taxa_tree_stats as stats};
        let mut conn = self.pool.get().await?;

        // Use the same join pattern as the species() method to find species for this source
        let taxa_datasets = diesel::alias!(datasets as taxa_datasets);

        let query = match filters.as_ref().and_then(|f| with_filters(f)) {
            Some(predicates) => species::table.filter(predicates).into_boxed(),
            None => species::table.into_boxed(),
        };

        let source_species: Vec<uuid::Uuid> = query
            .inner_join(taxon_names::table.on(species::id.eq(taxon_names::taxon_id)))
            .inner_join(attrs::table.on(attrs::name_id.eq(taxon_names::name_id)))
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .inner_join(taxa_datasets.on(taxa_datasets.field(datasets::id).eq(species::dataset_id)))
            .select(species::id)
            .distinct()
            .filter(datasets::source_id.eq(source.id))
            .filter(taxa_datasets.field(datasets::global_id).eq(ALA_DATASET_ID))
            .load::<uuid::Uuid>(&mut conn)
            .await?;

        if source_species.is_empty() {
            return Ok(vec![]);
        }

        // Then get stats for those species from the taxa tree stats (no cross-schema join needed)
        let summaries = stats::table
            .inner_join(taxa::table.on(taxa::id.eq(stats::id)))
            .select((
                taxa::scientific_name,
                taxa::canonical_name,
                stats::genomes,
                stats::loci,
                stats::specimens,
                stats::other,
                stats::total_genomic,
            ))
            .filter(stats::id.eq_any(source_species))
            .filter(taxa::rank.eq(arga_core::models::TaxonomicRank::Species))
            .distinct()
            .order(stats::loci.desc())
            .limit(10)
            .load::<super::taxa::Summary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    /// Summary statistics for a specific rank for species in this source
    pub async fn summary(
        &self,
        source: &Source,
        filters: &Option<Vec<Filter>>,
    ) -> Result<super::taxa::RankSummary, Error> {
        use schema::{datasets, name_attributes as attrs, taxa, taxon_names};
        use schema_gnl::{species, taxa_tree_stats as stats};

        let mut conn = self.pool.get().await?;

        // Use the exact same join pattern and logic as the species() method to ensure consistency
        let taxa_datasets = diesel::alias!(datasets as taxa_datasets);

        let query = match filters.as_ref().and_then(|f| with_filters(f)) {
            Some(predicates) => species::table.filter(predicates).into_boxed(),
            None => species::table.into_boxed(),
        };

        // Get the species IDs using the same query pattern as species() method
        let source_species: Vec<uuid::Uuid> = query
            .inner_join(taxon_names::table.on(species::id.eq(taxon_names::taxon_id)))
            .inner_join(attrs::table.on(attrs::name_id.eq(taxon_names::name_id)))
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .inner_join(taxa_datasets.on(taxa_datasets.field(datasets::id).eq(species::dataset_id)))
            .select(species::id)
            .distinct()
            .filter(datasets::source_id.eq(source.id))
            .filter(taxa_datasets.field(datasets::global_id).eq(ALA_DATASET_ID))
            .load::<uuid::Uuid>(&mut conn)
            .await?;

        let total_count = source_species.len() as i64;

        if total_count == 0 {
            return Ok(RankSummary {
                total: 0,
                loci: 0,
                genomes: 0,
                genomic_data: 0,
            });
        }

        // the taxa_tree_stats view summarises the total amount of data linked to descendants
        // of the supplied taxon. since we want the number of species that have data we instead
        // filter the taxa tree to species level nodes and count how many of them have a record
        // greater than zero. to get multiple values from one query we leverage the sum_if extension.
        let (genomes, loci, genomic_data) = stats::table
            .inner_join(taxa::table.on(taxa::id.eq(stats::id)))
            .filter(stats::taxon_id.eq_any(source_species))
            .filter(taxa::rank.eq(arga_core::models::TaxonomicRank::Species))
            .select((
                sum_if(stats::full_genomes.gt(BigDecimal::zero())).nullable(),
                sum_if(stats::loci.gt(BigDecimal::zero())).nullable(),
                sum_if(stats::total_genomic.gt(BigDecimal::zero())).nullable(),
            ))
            .get_result::<(Option<i64>, Option<i64>, Option<i64>)>(&mut conn)
            .await?;

        Ok(RankSummary {
            total: total_count,
            genomes: genomes.unwrap_or(0),
            loci: loci.unwrap_or(0),
            genomic_data: genomic_data.unwrap_or(0),
        })
    }

    // the top 10 latest genome releases for this source
    pub async fn latest_genome_releases(
        &self,
        source: &Source,
        filters: &Option<Vec<Filter>>,
    ) -> Result<Vec<GenomeRelease>, Error> {
        use schema::{datasets, deposition_events, name_attributes as attrs, taxa, taxon_names};
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        // Use the same join pattern as the species() method to find species for this source
        let taxa_datasets = diesel::alias!(datasets as taxa_datasets);

        let query = match filters.as_ref().and_then(|f| with_filters(f)) {
            Some(predicates) => species::table.filter(predicates).into_boxed(),
            None => species::table.into_boxed(),
        };

        let source_species: Vec<uuid::Uuid> = query
            .inner_join(taxon_names::table.on(species::id.eq(taxon_names::taxon_id)))
            .inner_join(attrs::table.on(attrs::name_id.eq(taxon_names::name_id)))
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .inner_join(taxa_datasets.on(taxa_datasets.field(datasets::id).eq(species::dataset_id)))
            .select(species::id)
            .distinct()
            .filter(datasets::source_id.eq(source.id))
            .filter(taxa_datasets.field(datasets::global_id).eq(ALA_DATASET_ID))
            .load::<uuid::Uuid>(&mut conn)
            .await?;

        if source_species.is_empty() {
            return Ok(vec![]);
        }

        // Step 2: Get deposition events for these species, ordered by event_date converted to proper date
        // Handle mixed date formats: DD/MM/YYYY, YYYY/MM/DD, D/M/YY, D/M/YYYY, D/MM/YY
        let genome_results = taxa::table
            .inner_join(taxon_names::table.on(taxa::id.eq(taxon_names::taxon_id)))
            .inner_join(schema::sequences::table.on(taxon_names::name_id.eq(schema::sequences::name_id)))
            .inner_join(deposition_events::table.on(schema::sequences::id.eq(deposition_events::sequence_id)))
            .select((taxa::scientific_name, taxa::canonical_name, deposition_events::event_date))
            .filter(taxa::id.eq_any(&source_species))
            .filter(taxa::rank.eq(arga_core::models::TaxonomicRank::Species))
            .filter(deposition_events::event_date.is_not_null())
            .order(DateParser::sql_date_order_converter().desc().nulls_last())
            .limit(10)
            .load::<(String, String, Option<String>)>(&mut conn)
            .await?;

        // Step 3: Parse the date strings into actual dates
        let summaries = genome_results
            .into_iter()
            .map(|(scientific_name, canonical_name, event_date)| {
                let parsed_date = event_date.and_then(|date_str| DateParser::parse_flexible_date(&date_str).ok());

                GenomeRelease {
                    scientific_name,
                    canonical_name,
                    release_date: parsed_date,
                }
            })
            .collect();

        Ok(summaries)
    }
}
