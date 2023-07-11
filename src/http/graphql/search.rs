use std::collections::HashMap;

use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::models::Taxon;
use crate::database::{schema, sum_if};
use crate::http::Error;
use crate::http::Context as State;
use crate::index::search::AssemblySummary;
use crate::index::search::{
    FullTextSearch,
    FullTextSearchItem,
    FullTextSearchResult,
    FullTextType,
    Classification,
};


pub struct Search;

#[Object]
impl Search {
    async fn full_text(&self, ctx: &Context<'_>, query: String, data_type: Option<String>) -> Result<FullTextSearchResult, Error> {
        use schema::{taxa, assemblies};

        let state = ctx.data::<State>().unwrap();
        let mut conn = state.database.pool.get().await?;

        let mut results = FullTextSearchResult::default();

        let data_type = data_type.unwrap_or("all".to_string());
        if data_type == "all" || data_type == "species" {
            let db_results = state.search.full_text(&query).await?;
            results.records.extend(db_results.records);
        };

        // get the taxon names to enrich the result data with
        let mut name_ids = Vec::with_capacity(results.records.len());
        for record in &results.records {
            match record {
                FullTextSearchItem::Taxon(item) => name_ids.push(item.name_id),
                _ => {},
            }
        }

        // look for taxonomic details for each name
        let rows = taxa::table
            .filter(taxa::name_id.eq_any(&name_ids))
            .load::<Taxon>(&mut conn)
            .await?;

        let mut record_map: HashMap<Uuid, Taxon> = HashMap::new();
        for row in rows {
            record_map.insert(row.name_id.clone(), row);
        }

        // get the total amounts of assembly records for each name
        let rows = assemblies::table
            .group_by(assemblies::name_id)
            .select((
                assemblies::name_id,
                sum_if(assemblies::refseq_category.eq("reference genome")),
                sum_if(assemblies::genome_rep.eq("Full")),
                sum_if(assemblies::genome_rep.eq("Partial")),
            ))
            .filter(assemblies::name_id.eq_any(&name_ids))
            .load::<(Uuid, i64, i64, i64)>(&mut conn)
            .await?;

        let mut assembly_map: HashMap<Uuid, (usize, usize, usize)> = HashMap::new();
        for row in rows {
            assembly_map.insert(row.0, (row.1 as usize, row.2 as usize, row.3 as usize));
        }


        // enrich the results with the gnl data
        for result in results.records.iter_mut() {
            match result {
                FullTextSearchItem::Taxon(item) => {
                    if let Some(record) = record_map.get(&item.name_id) {
                        item.scientific_name_authorship = record.species_authority.clone();
                        item.canonical_name = record.canonical_name.clone();
                        item.classification = Classification {
                            kingdom: record.kingdom.clone(),
                            phylum: record.phylum.clone(),
                            class: record.class.clone(),
                            order: record.order.clone(),
                            family: record.family.clone(),
                            genus: record.genus.clone(),
                        };
                    }

                    if let Some(summary) = assembly_map.get(&item.name_id) {
                        item.assembly_summary = AssemblySummary {
                            reference_genomes: summary.0,
                            whole_genomes: summary.1,
                            partial_genomes: summary.2,
                            barcodes: 0,
                        }
                    }
                }
                _ => {}
            };
        }


        // if we are only searching for species shortcut the request
        if data_type == "species" {
            return Ok(results);
        }


        // get the solr full text search results
        let solr_results = state.solr.full_text(&query).await?;
        results.records.extend(solr_results.records);

        // filter out the sequence types that wasn't requested
        results.records = results.records.into_iter().filter(|record| {
            data_type == "all" || data_type == match record {
                FullTextSearchItem::Taxon(_) => "species", // should already be filtered out if not 'all'
                FullTextSearchItem::GenomeSequence(item) => {
                    match item.r#type {
                        FullTextType::Taxon => "species",
                        FullTextType::ReferenceGenomeSequence => "whole_genomes",
                        FullTextType::WholeGenomeSequence => "whole_genomes",
                        FullTextType::PartialGenomeSequence => "partial_genomes",
                        FullTextType::UnknownGenomeSequence => "unknown_genomes",
                        FullTextType::Barcode => "barcodes",
                    }
                },
            }
        }).collect();

        // mix the results from multiple sources and rank them by the search score
        results.records.sort_by(|a, b| b.partial_cmp(a).unwrap());

        Ok(results)
    }
}
