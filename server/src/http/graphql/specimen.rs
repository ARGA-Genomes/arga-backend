use async_graphql::*;
use tracing::instrument;

use super::common::specimens::TissueDetails;
use super::common::{AccessionEvent, CollectionEvent, OrganismDetails};
use crate::database::{Database, models};
use crate::http::{Context as State, Error};


#[derive(OneofObject)]
pub enum SpecimenBy {
    EntityId(String),
    RecordId(String),
    SequenceRecordId(String),
    SequenceAccession(String),
}

#[derive(MergedObject)]
pub struct Specimen(SpecimenDetails, SpecimenQuery);

impl Specimen {
    pub async fn new(db: &Database, by: &SpecimenBy) -> Result<Specimen, Error> {
        let specimen = match by {
            SpecimenBy::EntityId(id) => db.specimens.find_by_id(&id).await?,
            SpecimenBy::RecordId(id) => db.specimens.find_by_record_id(&id).await?,
            SpecimenBy::SequenceRecordId(id) => db.specimens.find_by_sequence_record_id(&id).await?,
            SpecimenBy::SequenceAccession(id) => db.specimens.find_by_sequence_accession(&id).await?,
        };
        Ok(specimen.into())
    }

    pub fn from_record(specimen: models::Specimen) -> Specimen {
        let details = specimen.clone().into();
        let query = SpecimenQuery { specimen };
        Specimen(details, query)
    }
}

impl From<models::Specimen> for Specimen {
    fn from(value: models::Specimen) -> Self {
        Self::from_record(value)
    }
}


struct SpecimenQuery {
    specimen: models::Specimen,
}

#[Object]
impl SpecimenQuery {
    async fn canonical_name(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;
        let name = state.database.names.find_by_name_id(&self.specimen.name_id).await?;
        Ok(name.canonical_name)
    }

    async fn organism(&self, ctx: &Context<'_>) -> Result<OrganismDetails, Error> {
        let state = ctx.data::<State>()?;
        let organism = state.database.specimens.organism(&self.specimen.entity_id).await?;
        Ok(organism.into())
    }

    async fn collections(&self, ctx: &Context<'_>) -> Result<Vec<CollectionEvent>, Error> {
        let state = ctx.data::<State>()?;
        let specimen_id = &self.specimen.entity_id;
        let collections = state.database.specimens.collection_events(specimen_id).await?;
        Ok(collections.into_iter().map(|r| r.into()).collect())
    }

    async fn accessions(&self, ctx: &Context<'_>) -> Result<Vec<AccessionEvent>, Error> {
        let state = ctx.data::<State>()?;
        let specimen_id = &self.specimen.entity_id;
        let accessions = state.database.specimens.accession_events(specimen_id).await?;
        Ok(accessions.into_iter().map(|r| r.into()).collect())
    }

    async fn tissues(&self, ctx: &Context<'_>) -> Result<Vec<TissueDetails>, Error> {
        let state = ctx.data::<State>()?;
        let specimen_id = &self.specimen.entity_id;
        let tissues = state.database.specimens.tissues(specimen_id).await?;
        Ok(tissues.into_iter().map(|r| r.into()).collect())
    }

    async fn stats(&self, ctx: &Context<'_>) -> Result<SpecimenStats, Error> {
        let state = ctx.data::<State>()?;
        let specimen_id = &self.specimen.entity_id;
        let stats = state.database.specimens.stats(specimen_id).await?;
        Ok(stats.into())
    }

    #[instrument(skip(self, ctx))]
    async fn events(&self, ctx: &Context<'_>) -> Result<SpecimenEvents, Error> {
        let state = ctx.data::<State>()?;
        let specimen_id = &self.specimen.entity_id;
        let collections = state.database.specimens.collection_events(specimen_id).await?;
        let accessions = state.database.specimens.accession_events(specimen_id).await?;

        Ok(SpecimenEvents {
            collections: collections.into_iter().map(|r| r.into()).collect(),
            accessions: accessions.into_iter().map(|r| r.into()).collect(),
        })
    }
}


/// A specimen from a specific species.
#[derive(Clone, Debug, SimpleObject)]
pub struct SpecimenDetails {
    pub entity_id: String,
    pub organism_id: String,
    pub specimen_id: Option<String>,
}

impl From<models::Specimen> for SpecimenDetails {
    fn from(value: models::Specimen) -> Self {
        Self {
            entity_id: value.entity_id,
            organism_id: value.organism_id,
            specimen_id: value.specimen_id,
        }
    }
}


#[derive(SimpleObject)]
pub struct SpecimenEvents {
    collections: Vec<CollectionEvent>,
    accessions: Vec<AccessionEvent>,
}


#[derive(SimpleObject)]
pub struct SpecimenStats {
    pub entity_id: String,
    pub sequences: i64,
    pub whole_genomes: i64,
    pub loci: i64,
    pub other_genomic: i64,
    pub full_genomes: i64,
    pub partial_genomes: i64,
    pub complete_genomes: i64,
    pub assembly_chromosomes: i64,
    pub assembly_scaffolds: i64,
    pub assembly_contigs: i64,
}

impl From<models::SpecimenStats> for SpecimenStats {
    fn from(value: models::SpecimenStats) -> Self {
        Self {
            entity_id: value.entity_id,
            sequences: value.sequences,
            whole_genomes: value.whole_genomes,
            loci: value.loci,
            other_genomic: value.other_genomic,
            full_genomes: value.full_genomes,
            partial_genomes: value.partial_genomes,
            complete_genomes: value.complete_genomes,
            assembly_chromosomes: value.assembly_chromosomes,
            assembly_scaffolds: value.assembly_scaffolds,
            assembly_contigs: value.assembly_contigs,
        }
    }
}
