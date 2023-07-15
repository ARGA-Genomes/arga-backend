use async_graphql::*;
use serde::Deserialize;
use serde::Serialize;
use crate::database::Database;
use crate::http::Error;
use crate::http::Context as State;

use crate::database::models;
use crate::index::names::GetNames;


#[derive(MergedObject)]
pub struct Assembly(AssemblyDetails, AssemblyQuery);

impl Assembly {
    pub async fn new(db: &Database, accession: &str) -> Result<Assembly, Error> {
        let query = AssemblyQuery::new(db, accession).await?;
        Ok(Assembly(query.assembly.clone().into(), query))
    }
}


struct AssemblyQuery {
    assembly: models::Assembly,
}

#[Object]
impl AssemblyQuery {
    #[graphql(skip)]
    pub async fn new(db: &Database, accession: &str) -> Result<AssemblyQuery, Error> {
        Ok(AssemblyQuery {
            assembly: db.assembly.details(accession).await?,
        })
    }

    async fn canonical_name(&self, ctx: &Context<'_>) -> Result<Option<String>, Error> {
        let state = ctx.data::<State>().unwrap();
        let name = state.database.find_by_assembly_id(&self.assembly.id).await?;
        Ok(name.canonical_name)
    }

    /// Get the full assembly details
    async fn details(&self) -> AssemblyDetails {
        self.assembly.clone().into()
    }

    /// Get the assembly statistics associated with the provided assembly
    ///
    /// These stats are different to the stats used elsewhere throughout the backend,
    /// specifically they are imported data and reflect statistics about the assembly
    /// itself rather than stats about the arga index
    async fn stats(&self, ctx: &Context<'_>) -> Result<AssemblyDetailsStats, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.assembly.stats(&self.assembly.id).await.unwrap_or_default();
        Ok(stats.into())
    }

    /// Get all biosamples associated with the provided assembly
    async fn biosamples(&self, ctx: &Context<'_>) -> Result<Vec<BioSample>, Error> {
        let state = ctx.data::<State>().unwrap();

        let biosamples = match &self.assembly.biosample_id {
            Some(accession) => state.database.assembly.biosamples(accession).await?,
            None => vec![],
        };

        let biosamples = biosamples.into_iter().map(|r| r.into()).collect();
        Ok(biosamples)
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AssemblyDetails {
    pub id: String,
    pub accession: String,
    pub nuccore: Option<String>,
    pub refseq_category: Option<String>,
    pub specific_host: Option<String>,
    pub clone_strain: Option<String>,
    pub version_status: Option<String>,
    pub contam_screen_input: Option<String>,
    pub release_type: Option<String>,
    pub genome_rep: Option<String>,
    pub gbrs_paired_asm: Option<String>,
    pub paired_asm_comp: Option<String>,
    pub excluded_from_refseq: Option<String>,
    pub relation_to_type_material: Option<String>,
    pub asm_not_live_date: Option<String>,
    pub other_catalog_numbers: Option<String>,
    pub recorded_by: Option<String>,
    pub genetic_accession_uri: Option<String>,
    pub event_date: Option<String>,
}

impl From<models::Assembly> for AssemblyDetails {
    fn from(value: models::Assembly) -> Self {
        Self {
            id: value.id.to_string(),
            accession: value.accession,
            nuccore: value.nuccore,
            refseq_category: value.refseq_category,
            specific_host: value.specific_host,
            clone_strain: value.clone_strain,
            version_status: value.version_status,
            contam_screen_input: value.contam_screen_input,
            release_type: value.release_type,
            genome_rep: value.genome_rep,
            gbrs_paired_asm: value.gbrs_paired_asm,
            paired_asm_comp: value.paired_asm_comp,
            excluded_from_refseq: value.excluded_from_refseq,
            relation_to_type_material: value.relation_to_type_material,
            asm_not_live_date: value.asm_not_live_date,
            other_catalog_numbers: value.other_catalog_numbers,
            recorded_by: value.recorded_by,
            genetic_accession_uri: value.genetic_accession_uri,
            event_date: value.event_date,
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AssemblyDetailsStats {
    pub id: String,
    pub total_length: Option<i32>,
    pub spanned_gaps: Option<i32>,
    pub unspanned_gaps: Option<i32>,
    pub region_count: Option<i32>,
    pub scaffold_count: Option<i32>,
    pub scaffold_n50: Option<i32>,
    pub scaffold_l50: Option<i32>,
    pub scaffold_n75: Option<i32>,
    pub scaffold_n90: Option<i32>,
    pub contig_count: Option<i32>,
    pub contig_n50: Option<i32>,
    pub contig_l50: Option<i32>,
    pub total_gap_length: Option<i32>,
    pub molecule_count: Option<i32>,
    pub top_level_count: Option<i32>,
    pub component_count: Option<i32>,
    pub gc_perc: Option<i32>,
}

impl From<models::AssemblyStats> for AssemblyDetailsStats {
    fn from(value: models::AssemblyStats) -> Self {
        Self {
            id: value.id.to_string(),
            total_length: value.total_length,
            spanned_gaps: value.spanned_gaps,
            unspanned_gaps: value.unspanned_gaps,
            region_count: value.region_count,
            scaffold_count: value.scaffold_count,
            scaffold_n50: value.scaffold_n50,
            scaffold_l50: value.scaffold_l50,
            scaffold_n75: value.scaffold_n75,
            scaffold_n90: value.scaffold_n90,
            contig_count: value.contig_count,
            contig_n50: value.contig_n50,
            contig_l50: value.contig_l50,
            total_gap_length: value.total_gap_length,
            molecule_count: value.molecule_count,
            top_level_count: value.top_level_count,
            component_count: value.component_count,
            gc_perc: value.gc_perc,
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct BioSample {
    pub id: String,
    pub accession: String,

    pub sra: Option<String>,
    pub submission_date: Option<String>,
    pub publication_date: Option<String>,
    pub last_update: Option<String>,
    pub title: Option<String>,
    pub owner: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

impl From<models::BioSample> for BioSample {
    fn from(value: models::BioSample) -> Self {
        Self {
            id: value.id.to_string(),
            accession: value.accession,
            sra: value.sra,
            submission_date: value.submission_date.map(|d| d.to_string()),
            publication_date: value.publication_date.map(|d| d.to_string()),
            last_update: value.last_update.map(|d| d.to_string()),
            title: value.title,
            owner: value.owner,
            attributes: value.attributes,
        }
    }
}
