use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::index::genus::{GetGenus, Taxonomy};
use crate::database::sum_if;
use super::{schema, schema_gnl, Database, Error, Taxon, PgPool, models::RankedTaxon};


#[async_trait]
impl GetGenus for Database {
    type Error = Error;

    async fn taxonomy(&self, name: &str) -> Result<Taxonomy, Error> {
        use schema_gnl::gnl::dsl::*;
        let mut conn = self.pool.get().await?;

        let taxon = gnl
            .select((
                scientific_name_authorship,
                canonical_name,
                kingdom,
                phylum,
                class,
                order,
                family,
                genus,
            ))
            .filter(taxon_rank.eq("genus"))
            .filter(canonical_name.eq(name))
            .first::<Taxon>(&mut conn).await?;

        Ok(Taxonomy::from(taxon))
    }
}


#[derive(Clone)]
pub struct GenusProvider {
    pub pool: PgPool,
}

impl GenusProvider {
    pub async fn species(&self, genus_name: &str) -> Result<Vec<RankedTaxon>, Error> {
        use schema_gnl::ranked_taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let species = ranked_taxa
            .filter(genus.eq(genus_name))
            // .filter(taxon_rank.eq("species"))
            .order_by(taxa_priority)
            .load::<RankedTaxon>(&mut conn)
            .await?;

        Ok(species)
    }

    pub async fn species_summary(&self, name_ids: &Vec<uuid::Uuid>) -> Result<Vec<SpeciesSummary>, Error> {
        use schema::assemblies;
        let mut conn = self.pool.get().await?;

        let rows = assemblies::table
            .group_by(assemblies::name_id)
            .select((
                assemblies::name_id,
                sum_if(assemblies::refseq_category.eq("reference genome")),
                sum_if(assemblies::genome_rep.eq("Full")),
                sum_if(assemblies::genome_rep.eq("Partial")),
            ))
            .filter(assemblies::name_id.eq_any(name_ids))
            .load::<(uuid::Uuid, i64, i64, i64)>(&mut conn)
            .await?;

        let mut summaries = Vec::new();
        for (name_id, reference_genomes, whole_genomes, partial_genomes) in rows {
            summaries.push(SpeciesSummary {
                name_id,
                reference_genomes: reference_genomes as usize,
                whole_genomes: whole_genomes as usize,
                partial_genomes: partial_genomes as usize,
                mitogenomes: 0,
                barcodes: 0,
                other: 0,
            });
        }

        Ok(summaries)
    }
}


pub struct SpeciesSummary {
    pub name_id: uuid::Uuid,
    pub reference_genomes: usize,
    pub whole_genomes: usize,
    pub partial_genomes: usize,
    pub mitogenomes: usize,
    pub barcodes: usize,
    pub other: usize,
}
