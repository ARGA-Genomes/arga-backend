use std::collections::HashMap;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::models::TaxonomicStatus;
use crate::database::sum_if;
use crate::http::graphql::common::Taxonomy;
use super::{schema, Error, PgPool};
use super::models::Taxon;


#[derive(Clone)]
pub struct GenusProvider {
    pub pool: PgPool,
}

impl GenusProvider {
    pub async fn taxonomy(&self, name: &str) -> Result<Taxonomy, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let taxon = taxa
            .filter(genus.eq(name))
            .first::<Taxon>(&mut conn).await?;

        Ok(Taxonomy::from(taxon))
    }

    pub async fn species(&self, genus_name: &str) -> Result<Vec<Taxon>, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let species = taxa
            .filter(genus.eq(genus_name))
            .filter(status.eq_any(&[TaxonomicStatus::Valid, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .load::<Taxon>(&mut conn)
            .await?;

        Ok(species)
    }

    pub async fn species_summary(&self, name_ids: &Vec<uuid::Uuid>) -> Result<Vec<SpeciesSummary>, Error> {
        use schema::{assemblies, markers};
        let mut conn = self.pool.get().await?;

        let mut map = HashMap::new();

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

        for (name_id, reference_genomes, whole_genomes, partial_genomes) in rows {
            map.insert(name_id.clone(), SpeciesSummary {
                name_id,
                reference_genomes: reference_genomes as usize,
                whole_genomes: whole_genomes as usize,
                partial_genomes: partial_genomes as usize,
                organelles: 0,
                barcodes: 0,
                other: 0,
            });
        }


        let rows = markers::table
            .group_by(markers::name_id)
            .select((
                markers::name_id,
                sum_if(markers::accession.is_not_null()),
            ))
            .filter(markers::name_id.eq_any(name_ids))
            .load::<(uuid::Uuid, i64)>(&mut conn)
            .await?;

        for (name_id, markers) in rows {
            let entry = map.entry(name_id.clone());
            entry.or_default().barcodes = markers as usize;
        }


        let summaries = Vec::from_iter(map.values().cloned());
        Ok(summaries)
    }
}


#[derive(Clone, Default)]
pub struct SpeciesSummary {
    pub name_id: uuid::Uuid,
    pub reference_genomes: usize,
    pub whole_genomes: usize,
    pub partial_genomes: usize,
    pub organelles: usize,
    pub barcodes: usize,
    pub other: usize,
}
