use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::models::{Assembly, AssemblyStats, BioSample};
use crate::index::assembly::{self, GetAssembly, GetAssemblyStats, GetBioSamples};
use super::{schema, Database, Error};


#[async_trait]
impl GetAssembly for Database {
    type Error = Error;

    async fn get_assembly(&self, accession: &str) -> Result<assembly::AssemblyDetails, Self::Error> {
        use schema::assemblies;
        let mut conn = self.pool.get().await?;

        let assembly = assemblies::table
            .filter(assemblies::accession.eq(accession))
            .get_result::<Assembly>(&mut conn)
            .await?;

        Ok(assembly.into())
    }
}

impl From<Assembly> for assembly::AssemblyDetails {
    fn from(value: Assembly) -> Self {
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


#[async_trait]
impl GetAssemblyStats for Database {
    type Error = Error;

    async fn get_assembly_stats(&self, assembly_id: &Uuid) -> Result<assembly::AssemblyStats, Self::Error> {
        use schema::assembly_stats;
        let mut conn = self.pool.get().await?;

        let stat = assembly_stats::table
            .filter(assembly_stats::assembly_id.eq(assembly_id))
            .get_result::<AssemblyStats>(&mut conn)
            .await?;

        Ok(stat.into())
    }
}

impl From<AssemblyStats> for assembly::AssemblyStats {
    fn from(value: AssemblyStats) -> Self {
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


#[async_trait]
impl GetBioSamples for Database {
    type Error = Error;

    async fn get_biosamples(&self, accession: &str) -> Result<Vec<assembly::BioSample>, Self::Error> {
        use schema::biosamples;
        let mut conn = self.pool.get().await?;

        let records = biosamples::table
            .filter(biosamples::accession.eq(accession))
            .load::<BioSample>(&mut conn)
            .await?;

        let records = records.into_iter().map(|r| r.into()).collect();
        Ok(records)
    }
}

impl From<BioSample> for assembly::BioSample {
    fn from(value: BioSample) -> Self {
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
