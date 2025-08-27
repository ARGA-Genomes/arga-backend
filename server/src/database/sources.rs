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

#[derive(Clone, Debug)]
pub enum SummarySort {
    TotalGenomic,
    Genomes,
    Loci,
}

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

#[derive(Clone, Debug)]
pub struct KingdomPhylumCount {
    pub kingdom: String,
    pub phylum: String,
    pub count: i64,
}


impl SourceProvider {
    // Helper method to handle NotFound errors
    fn handle_not_found<T>(result: Result<T, diesel::result::Error>, identifier: &str) -> Result<T, Error> {
        match result {
            Ok(value) => Ok(value),
            Err(diesel::result::Error::NotFound) => Err(Error::NotFound(identifier.to_string())),
            Err(err) => Err(Error::Connection(err)),
        }
    }

    // Helper method to get source species IDs with consistent filtering
    async fn get_source_species_ids(&self, source: &Source, filters: &Option<Vec<Filter>>) -> Result<Vec<Uuid>, Error> {
        use schema::{datasets, name_attributes as attrs, taxon_names};
        use schema_gnl::species;

        let mut conn = self.pool.get().await?;

        let query = match filters.as_ref().and_then(|f| with_filters(f)) {
            Some(predicates) => species::table.filter(predicates).into_boxed(),
            None => species::table.into_boxed(),
        };

        let taxa_datasets = diesel::alias!(datasets as taxa_datasets);

        let source_species = query
            .inner_join(taxon_names::table.on(species::id.eq(taxon_names::taxon_id)))
            .inner_join(attrs::table.on(attrs::name_id.eq(taxon_names::name_id)))
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .inner_join(taxa_datasets.on(taxa_datasets.field(datasets::id).eq(species::dataset_id)))
            .select(species::id)
            .distinct()
            .filter(datasets::source_id.eq(source.id))
            .filter(taxa_datasets.field(datasets::global_id).eq(ALA_DATASET_ID))
            .load::<Uuid>(&mut conn)
            .await?;

        Ok(source_species)
    }

    // generic method to get species summaries with different sorting
    async fn get_species_summary(
        &self,
        source: &Source,
        filters: &Option<Vec<Filter>>,
        sort: SummarySort,
        limit: i64,
    ) -> Result<Vec<super::taxa::Summary>, Error> {
        use schema::taxa;
        use schema_gnl::taxa_tree_stats as stats;

        let source_species = self.get_source_species_ids(source, filters).await?;

        if source_species.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self.pool.get().await?;

        let summaries = match sort {
            SummarySort::TotalGenomic => {
                stats::table
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
                    .filter(stats::taxon_id.eq_any(&source_species))
                    .distinct()
                    .order(stats::total_genomic.desc())
                    .limit(limit)
                    .load::<super::taxa::Summary>(&mut conn)
                    .await?
            }
            SummarySort::Genomes => {
                stats::table
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
                    .filter(stats::taxon_id.eq_any(&source_species))
                    .distinct()
                    .order(stats::genomes.desc())
                    .limit(limit)
                    .load::<super::taxa::Summary>(&mut conn)
                    .await?
            }
            SummarySort::Loci => {
                stats::table
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
                    .filter(stats::id.eq_any(&source_species))
                    .distinct()
                    .order(stats::loci.desc())
                    .limit(limit)
                    .load::<super::taxa::Summary>(&mut conn)
                    .await?
            }
        };

        Ok(summaries)
    }

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

        Self::handle_not_found(source, &id.to_string())
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Source, Error> {
        use schema::sources;
        let mut conn = self.pool.get().await?;

        let source = sources::table
            .filter(sources::name.eq(name))
            .get_result::<Source>(&mut conn)
            .await;

        Self::handle_not_found(source, name)
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

        let query = match with_filters(filters) {
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

    // the top 10 species with the most total genomic data for this source
    pub async fn species_genomic_data_summary(
        &self,
        source: &Source,
        filters: &Option<Vec<Filter>>,
    ) -> Result<Vec<super::taxa::Summary>, Error> {
        self.get_species_summary(source, filters, SummarySort::TotalGenomic, 10)
            .await
    }

    // the top 10 species with the most genomes for this source
    pub async fn species_genomes_summary(
        &self,
        source: &Source,
        filters: &Option<Vec<Filter>>,
    ) -> Result<Vec<super::taxa::Summary>, Error> {
        self.get_species_summary(source, filters, SummarySort::Genomes, 10)
            .await
    }

    // the top 10 species with the most loci for this source
    pub async fn species_loci_summary(
        &self,
        source: &Source,
        filters: &Option<Vec<Filter>>,
    ) -> Result<Vec<super::taxa::Summary>, Error> {
        self.get_species_summary(source, filters, SummarySort::Loci, 10).await
    }

    // Summary statistics for a specific rank for species in this source
    pub async fn summary(
        &self,
        source: &Source,
        filters: &Option<Vec<Filter>>,
    ) -> Result<super::taxa::RankSummary, Error> {
        use schema::taxa;
        use schema_gnl::taxa_tree_stats as stats;

        let source_species = self.get_source_species_ids(source, filters).await?;

        let total_count = source_species.len() as i64;

        if total_count == 0 {
            return Ok(RankSummary {
                total: 0,
                loci: 0,
                genomes: 0,
                genomic_data: 0,
            });
        }

        let mut conn = self.pool.get().await?;

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
        use schema::{datasets, deposition_events, name_attributes as attrs, names, sequences, taxon_names};
        use schema_gnl::species;

        let mut conn = self.pool.get().await?;

        let query = match filters.as_ref().and_then(|f| with_filters(f)) {
            Some(predicates) => species::table.filter(predicates).into_boxed(),
            None => species::table.into_boxed(),
        };

        let taxa_datasets = diesel::alias!(datasets as taxa_datasets);

        let source_names = query
            .inner_join(taxon_names::table.on(species::id.eq(taxon_names::taxon_id)))
            .inner_join(attrs::table.on(attrs::name_id.eq(taxon_names::name_id)))
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .inner_join(taxa_datasets.on(taxa_datasets.field(datasets::id).eq(species::dataset_id)))
            .select(attrs::name_id)
            .distinct()
            .filter(datasets::source_id.eq(source.id))
            .filter(taxa_datasets.field(datasets::global_id).eq(ALA_DATASET_ID))
            .load::<Uuid>(&mut conn)
            .await?;

        if source_names.is_empty() {
            return Ok(vec![]);
        }

        // Get deposition events for these species, ordered by event_date converted to proper date
        // Handle mixed date formats: DD/MM/YYYY, YYYY/MM/DD, D/M/YY, D/M/YYYY, D/MM/YY
        let genome_results = sequences::table
            .inner_join(deposition_events::table.on(sequences::id.eq(deposition_events::sequence_id)))
            .inner_join(names::table.on(names::id.eq(sequences::name_id)))
            .select((names::scientific_name, names::canonical_name, deposition_events::event_date))
            .filter(names::id.eq_any(source_names))
            .filter(deposition_events::event_date.is_not_null())
            .order(DateParser::sql_date_order_converter().desc().nulls_last())
            .limit(10)
            .load::<(String, String, Option<String>)>(&mut conn)
            .await?;

        // Parse the date strings into actual dates
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

    pub async fn taxonomic_diversity(
        &self,
        source: &Source,
        filters: &Option<Vec<Filter>>,
    ) -> Result<Vec<KingdomPhylumCount>, Error> {
        use std::collections::HashMap;

        use schema::{datasets, name_attributes as attrs, taxon_names};
        use schema_gnl::species;

        let mut conn = self.pool.get().await?;

        let query = match filters.as_ref().and_then(|f| with_filters(f)) {
            Some(predicates) => species::table.filter(predicates).into_boxed(),
            None => species::table.into_boxed(),
        };

        let taxa_datasets = diesel::alias!(datasets as taxa_datasets);

        let source_species = query
            .inner_join(taxon_names::table.on(species::id.eq(taxon_names::taxon_id)))
            .inner_join(attrs::table.on(attrs::name_id.eq(taxon_names::name_id)))
            .inner_join(datasets::table.on(datasets::id.eq(attrs::dataset_id)))
            .inner_join(taxa_datasets.on(taxa_datasets.field(datasets::id).eq(species::dataset_id)))
            .select(species::classification)
            .distinct()
            .filter(datasets::source_id.eq(source.id))
            .filter(taxa_datasets.field(datasets::global_id).eq(ALA_DATASET_ID))
            .load::<serde_json::Value>(&mut conn)
            .await?;

        // Group by kingdom and count unique phylums
        let mut kingdom_phylum_counts: HashMap<String, HashMap<String, i64>> = HashMap::new();

        for classification in source_species {
            if let (Some(kingdom), Some(phylum)) = (
                classification.get("kingdom").and_then(|v| v.as_str()),
                classification.get("phylum").and_then(|v| v.as_str()),
            ) {
                let kingdom_str = kingdom.to_string();
                let phylum_str = phylum.to_string();

                kingdom_phylum_counts
                    .entry(kingdom_str)
                    .or_insert_with(HashMap::new)
                    .entry(phylum_str)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }

        // Convert to result format
        let mut results = Vec::new();
        for (kingdom, phylum_counts) in kingdom_phylum_counts {
            for (phylum, count) in phylum_counts {
                results.push(KingdomPhylumCount {
                    kingdom: kingdom.clone(),
                    phylum,
                    count,
                });
            }
        }

        // Sort by kingdom, then by phylum for consistent ordering
        results.sort_by(|a, b| a.kingdom.cmp(&b.kingdom).then_with(|| a.phylum.cmp(&b.phylum)));

        Ok(results)
    }
}
