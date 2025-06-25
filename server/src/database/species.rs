use arga_core::models::Species;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::Queryable;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::extensions::Paginate;
use super::models::{
    GenomicComponent,
    Marker,
    Name,
    NameAttribute,
    RegionType,
    Regions,
    Taxon,
    TaxonName,
    TaxonPhoto,
    VernacularName,
    WholeGenome,
};
use super::{schema, schema_gnl, Error, PageResult, PgPool};
use crate::database::extensions::{date_part, lower_opt, sum_if, whole_genome_filters};

const NCBI_REFSEQ_DATASET_ID: &str = "ARGA:TL:0002002";

#[derive(Debug, Clone, Default, Queryable, Serialize, Deserialize)]
pub struct Summary {
    pub id: Uuid,
    pub genomes: i64,
    pub loci: i64,
    pub specimens: i64,
    pub other: i64,
    pub total_genomic: i64,
}

#[derive(Debug, Clone, Default, Queryable, Serialize, Deserialize)]
pub struct MarkerSummary {
    pub name_id: Uuid,
    pub barcodes: i64,
}

#[derive(Debug, Queryable)]
pub struct SpecimenSummary {
    pub entity_id: String,
    pub collection_repository_id: Option<String>,
    pub collection_repository_code: Option<String>,
    pub institution_code: Option<String>,
    pub institution_name: Option<String>,
    pub type_status: Option<String>,
    pub locality: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub sequences: i64,
    pub whole_genomes: i64,
    pub markers: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSummary {
    pub genomes: Option<i64>,
    pub loci: Option<i64>,
    pub specimens: Option<i64>,
    pub other: Option<i64>,
    pub total_genomic: Option<i64>,
}

#[derive(Debug, Clone, Default, Queryable, Serialize, Deserialize)]
pub struct SpecimensOverview {
    pub total: i64,
    pub major_collections: Vec<String>,
    pub holotype: Option<String>,
    pub other_types: i64,
    pub formal_vouchers: i64,
    pub tissues: i64,
    pub genomic_dna: i64,
    pub australian_material: i64,
    pub non_australian_material: i64,
    pub collection_years: Vec<(i64, i64)>,
}

#[derive(Clone)]
pub struct SpeciesProvider {
    pub pool: PgPool,
}

impl SpeciesProvider {
    /// Get taxonomic information for a specific species.
    pub async fn taxonomy(&self, names: &Vec<Name>) -> Result<Vec<Species>, Error> {
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let taxa = TaxonName::belonging_to(names)
            .inner_join(species::table)
            .select(species::all_columns)
            .load(&mut conn)
            .await?;

        Ok(taxa)
    }

    pub async fn vernacular_names(&self, name_ids: &Vec<Uuid>) -> Result<Vec<VernacularName>, Error> {
        use schema::vernacular_names::dsl::*;
        let mut conn = self.pool.get().await?;

        let names = vernacular_names
            .filter(name_id.eq_any(name_ids))
            .load::<VernacularName>(&mut conn)
            .await?;

        Ok(names)
    }

    pub async fn synonyms(&self, _name_id: &Uuid) -> Result<Vec<Taxon>, Error> {
        // use schema::{taxa, taxon_history};
        // let mut conn = self.pool.get().await?;

        // let (old_taxa, new_taxa) = diesel::alias!(taxa as old_taxa, taxa as new_taxa);

        // FIXME: determine synonyms based on a taxonomic system and taxon_names
        let synonyms = vec![];
        // let synonyms = taxon_history::table
        //     .inner_join(old_taxa.on(taxon_history::old_taxon_id.eq(old_taxa.field(taxa::id))))
        //     .inner_join(new_taxa.on(taxon_history::new_taxon_id.eq(new_taxa.field(taxa::id))))
        //     .filter(new_taxa.field(taxa::name_id).eq(name_id))
        //     .select(old_taxa.fields(taxa::all_columns))
        //     .load::<Taxon>(&mut conn)
        //     .await?;

        Ok(synonyms)
    }

    pub async fn summary(&self, ids: &Vec<Uuid>) -> Result<Vec<Summary>, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        // get the data summaries for each species record
        let summaries = species
            .select((id, genomes, loci, specimens, other, total_genomic))
            .filter(id.eq_any(ids))
            .load::<Summary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn marker_summary(&self, ids: &Vec<Uuid>) -> Result<Vec<MarkerSummary>, Error> {
        use schema_gnl::markers::dsl::*;
        let mut conn = self.pool.get().await?;

        // get the total amounts of assembly records for each name
        let summaries = markers
            .group_by(name_id)
            .select((name_id, diesel::dsl::count_star()))
            .filter(name_id.eq_any(ids))
            .load::<MarkerSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn specimens(&self, names: &Vec<Name>, page: i64, page_size: i64) -> PageResult<SpecimenSummary> {
        use schema::{accession_events, collection_events, specimens};
        use schema_gnl::specimen_stats;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = specimens::table
            .inner_join(specimen_stats::table)
            .left_join(collection_events::table)
            .left_join(accession_events::table)
            .select((
                specimens::entity_id,
                accession_events::collection_repository_id.nullable(),
                accession_events::collection_repository_code.nullable(),
                accession_events::institution_code.nullable(),
                accession_events::institution_name.nullable(),
                accession_events::type_status.nullable(),
                collection_events::locality.nullable(),
                collection_events::country.nullable(),
                collection_events::latitude.nullable(),
                collection_events::longitude.nullable(),
                specimen_stats::sequences,
                specimen_stats::whole_genomes,
                specimen_stats::markers,
            ))
            .filter(specimens::name_id.eq_any(name_ids))
            .order((
                accession_events::type_status.asc(),
                specimen_stats::sequences.desc(),
                accession_events::institution_code.asc(),
                accession_events::collection_repository_id.asc(),
                specimens::entity_id.asc(),
            ))
            .paginate(page)
            .per_page(page_size)
            .load::<(SpecimenSummary, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn whole_genomes(
        &self,
        names: &Vec<Name>,
        filters: &Vec<whole_genome_filters::Filter>,
        page: i64,
        page_size: i64,
    ) -> PageResult<WholeGenome> {
        use schema_gnl::whole_genomes;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let mut query = whole_genomes::table
            .filter(whole_genomes::name_id.eq_any(name_ids))
            .into_boxed();

        if let Some(expr) = whole_genome_filters::with_filters(&filters) {
            query = query.filter(expr);
        }

        let records = query
            .order(whole_genomes::accession)
            .paginate(page)
            .per_page(page_size)
            .load::<(WholeGenome, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn loci(&self, names: &Vec<Name>, page: i64, page_size: i64) -> PageResult<Marker> {
        use schema_gnl::markers;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = markers::table
            .filter(markers::name_id.eq_any(name_ids))
            .order(markers::accession)
            .paginate(page)
            .per_page(page_size)
            .load::<(Marker, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn genomic_components(
        &self,
        names: &Vec<Name>,
        page: i64,
        page_size: i64,
    ) -> PageResult<GenomicComponent> {
        use schema_gnl::genomic_components;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = genomic_components::table
            .filter(genomic_components::name_id.eq_any(name_ids))
            .order(genomic_components::accession)
            .paginate(page)
            .per_page(page_size)
            .load::<(GenomicComponent, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn reference_genome(&self, names: &Vec<Name>) -> Result<Option<WholeGenome>, Error> {
        use schema::datasets;
        use schema_gnl::whole_genomes;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let record = whole_genomes::table
            .inner_join(datasets::table)
            .select(whole_genomes::all_columns)
            .filter(whole_genomes::name_id.eq_any(name_ids))
            .filter(datasets::global_id.eq(NCBI_REFSEQ_DATASET_ID))
            .get_result::<WholeGenome>(&mut conn)
            .await
            .optional()?;

        Ok(record)
    }

    pub async fn attributes(&self, names: &Vec<Name>) -> Result<Vec<NameAttribute>, Error> {
        use schema::name_attributes;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = name_attributes::table
            .filter(name_attributes::name_id.eq_any(name_ids))
            .load::<NameAttribute>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn regions_ibra(&self, names: &Vec<Name>) -> Result<Vec<Regions>, Error> {
        use schema::regions;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let regions = regions::table
            .filter(regions::name_id.eq_any(name_ids))
            .filter(regions::region_type.eq(RegionType::Ibra))
            .load::<Regions>(&mut conn)
            .await?;

        Ok(regions)
    }

    pub async fn regions_imcra(&self, names: &Vec<Name>) -> Result<Vec<Regions>, Error> {
        use schema::regions;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let regions = regions::table
            .filter(regions::name_id.eq_any(name_ids))
            .filter(regions::region_type.eq(RegionType::Imcra))
            .load::<Regions>(&mut conn)
            .await?;

        Ok(regions)
    }

    pub async fn photos(&self, names: &Vec<Name>) -> Result<Vec<TaxonPhoto>, Error> {
        use schema::{taxon_names, taxon_photos};
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let photos = taxon_photos::table
            .inner_join(taxon_names::table.on(taxon_names::taxon_id.eq(taxon_photos::taxon_id)))
            .select(taxon_photos::all_columns)
            .filter(taxon_names::name_id.eq_any(name_ids))
            .load::<TaxonPhoto>(&mut conn)
            .await?;

        Ok(photos)
    }

    pub async fn data_summary(&self, name_ids: &Vec<Uuid>) -> Result<DataSummary, Error> {
        use diesel::dsl::sum;
        use schema_gnl::name_data_summaries;

        let mut conn = self.pool.get().await?;

        let (genomes, loci, specimens, other, total_genomic) = name_data_summaries::table
            .select((
                sum(name_data_summaries::genomes),
                sum(name_data_summaries::markers),
                sum(name_data_summaries::specimens),
                sum(name_data_summaries::other),
                sum(name_data_summaries::total_genomic),
            ))
            .filter(name_data_summaries::name_id.eq_any(name_ids))
            .get_result::<(Option<i64>, Option<i64>, Option<i64>, Option<i64>, Option<i64>)>(&mut conn)
            .await?;

        Ok(DataSummary {
            genomes,
            loci,
            specimens,
            other,
            total_genomic,
        })
    }

    pub async fn specimens_overview(&self, name_ids: &Vec<Uuid>) -> Result<SpecimensOverview, Error> {
        use diesel::dsl::{count_distinct, count_star};
        use schema::{accession_events, collection_events, specimens};

        let mut conn = self.pool.get().await?;

        let (institution, id) = accession_events::table
            .inner_join(specimens::table)
            .filter(specimens::name_id.eq_any(name_ids))
            .filter(lower_opt(accession_events::type_status.nullable()).eq("holotype"))
            .select((accession_events::institution_code, accession_events::collection_repository_id))
            .get_result::<(Option<String>, Option<String>)>(&mut conn)
            .await?;

        let holotype = match (institution, id) {
            (Some(institution), Some(id)) => Some(format!("{institution} {id}")),
            (None, Some(id)) => Some(format!("{id}")),
            _ => None,
        };

        // we can get the stats for a small amount of names without requiring a materialized view as it
        // is fast enough. importantly note that we don't check for null for columns that don't match
        // a value; this is because a null compared with a value is always false, so they aren't counted
        // in the sum_if helper
        let (total, other_types, formal_vouchers, australian_material, non_australian_material) = specimens::table
            .left_join(accession_events::table)
            .left_join(collection_events::table)
            .filter(specimens::name_id.eq_any(name_ids))
            .select((
                count_distinct(specimens::entity_id),
                sum_if(lower_opt(accession_events::type_status).nullable().ne("holotype")),
                sum_if(accession_events::collection_repository_id.nullable().is_not_null()),
                sum_if(lower_opt(collection_events::country).nullable().eq("australia")),
                sum_if(lower_opt(collection_events::country).nullable().ne("australia")),
            ))
            .get_result::<(i64, i64, i64, i64, i64)>(&mut conn)
            .await?;

        let major_collections = specimens::table
            .inner_join(accession_events::table)
            .filter(specimens::name_id.eq_any(name_ids))
            .filter(accession_events::institution_code.is_not_null())
            .group_by(accession_events::institution_code)
            .select(accession_events::institution_code.assume_not_null())
            .order(count_star().desc())
            .limit(4)
            .load::<String>(&mut conn)
            .await?;

        // NOTE: This is not ideal but diesel doesn't have support for aliased expressions yet
        // so we can't group by an extracted date and then select it
        let event_date_year =
            sql::<diesel::sql_types::BigInt>(r#"extract(YEAR FROM event_date)::bigint AS event_date_year"#);

        let collection_years = specimens::table
            .inner_join(collection_events::table)
            .filter(specimens::name_id.eq_any(name_ids))
            .filter(collection_events::event_date.is_not_null())
            .group_by(sql::<diesel::sql_types::BigInt>("event_date_year"))
            .select((event_date_year, count_star()))
            .load::<(i64, i64)>(&mut conn)
            .await?;

        Ok(SpecimensOverview {
            total,
            major_collections,
            holotype,
            other_types,
            formal_vouchers,
            tissues: 0,
            genomic_dna: 0,
            australian_material,
            non_australian_material,
            collection_years,
        })
    }

    pub async fn specimens_map_markers(&self, name_ids: &Vec<Uuid>) -> Result<Vec<SpecimenMapMarker>, Error> {
        use schema::{accession_events, collection_events, specimens};

        let mut conn = self.pool.get().await?;

        let records = specimens::table
            .inner_join(collection_events::table)
            .inner_join(accession_events::table)
            .filter(specimens::name_id.eq_any(name_ids))
            .filter(collection_events::latitude.is_not_null())
            .filter(collection_events::longitude.is_not_null())
            .select((
                accession_events::collection_repository_id,
                accession_events::type_status,
                collection_events::latitude.assume_not_null(),
                collection_events::longitude.assume_not_null(),
            ))
            .load::<SpecimenMapMarker>(&mut conn)
            .await?;

        Ok(records)
    }
}

#[derive(Debug, Queryable)]
pub struct SpecimenMapMarker {
    pub collection_repository_id: Option<String>,
    pub type_status: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
}


// #[derive(Queryable, Debug)]
// struct Distribution {
//     pub locality: Option<String>,
//     pub country: Option<String>,
//     pub country_code: Option<String>,
//     pub threat_status: Option<String>,
//     pub source: Option<String>,
// }

// impl From<Distribution> for species::Distribution {
//     fn from(source: Distribution) -> Self {
//         Self {
//             locality: source.locality,
//             country: source.country,
//             country_code: source.country_code,
//             threat_status: source.threat_status,
//             source: source.source,
//         }
//     }
// }

// #[async_trait]
// impl GetRegions for Database {
//     type Error = Error;

//     async fn ibra(&self, name: &Name) -> Result<Vec<species::Region>, Error> {
//         use schema::regions;
//         let mut conn = self.pool.get().await?;

//         let regions = regions::table
//             .select(regions::values)
//             .filter(regions::name_id.eq(name.id))
//             .filter(regions::region_type.eq(RegionType::Ibra))
//             .load::<Vec<Option<String>>>(&mut conn)
//             .await?;

//         let mut filtered = Vec::new();
//         for region in regions.concat() {
//             if let Some(name) = region {
//                 filtered.push(species::Region { name });
//             }
//         }

//         filtered.sort();
//         filtered.dedup();
//         Ok(filtered)
//     }

//     async fn imcra(&self, name: &Name) -> Result<Vec<species::Region>, Error> {
//         use schema::regions;
//         let mut conn = self.pool.get().await?;

//         let regions = regions::table
//             .select(regions::values)
//             .filter(regions::name_id.eq(name.id))
//             .filter(regions::region_type.eq(RegionType::Imcra))
//             .load::<Vec<Option<String>>>(&mut conn)
//             .await?;

//         let mut filtered = Vec::new();
//         for region in regions.concat() {
//             if let Some(name) = region {
//                 filtered.push(species::Region { name });
//             }
//         }

//         filtered.sort();
//         filtered.dedup();
//         Ok(filtered)
//     }
// }
