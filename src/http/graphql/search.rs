use std::collections::HashMap;

use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::models::{Species, TaxonomicStatus, Taxon};
use crate::database::{schema, schema_gnl, sum_if};
use crate::http::Error;
use crate::http::Context as State;
use crate::index::providers::search::SearchItem;
use crate::index::search::{AssemblySummary, TaxonItem, GenusItem};
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
        use schema::{assemblies, taxa};
        use schema_gnl::species;

        let state = ctx.data::<State>().unwrap();
        let mut conn = state.database.pool.get().await?;

        let mut search_results = Vec::new();

        let data_type = data_type.unwrap_or("all".to_string());
        if data_type == "all" || data_type == "species" {
            let db_results = state.search.species(&query)?;
            search_results.extend(db_results);
        };

        // get the taxon details to enrich the result data with
        let mut name_ids: Vec<Uuid> = Vec::new();
        let mut genera_names: Vec<String> = Vec::new();

        for record in &search_results {
            match record {
                SearchItem::Species { uuid, .. } => name_ids.push(uuid.clone()),
                SearchItem::UndescribedSpecies { genus, .. } => genera_names.push(genus.into()),
                _ => {},
            }
        }

        // look for undescribed species in found genera
        let rows = taxa::table
            .filter(taxa::genus.eq_any(genera_names))
            .filter(taxa::status.eq_any([TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .load::<Taxon>(&mut conn)
            .await?;

        let mut undescribed_map: HashMap<String, Vec<Taxon>> = HashMap::new();
        for row in rows {
            // we also add the undescribed species name id into the map
            // so that we can load additional information like assembly counts
            name_ids.push(row.name_id.clone());

            if let Some(genus) = row.genus.to_owned() {
                let entry = undescribed_map.entry(genus);
                entry.or_default().push(row);
            }
        }

        // look for taxonomic details for each name
        let rows = species::table
            .filter(species::name_id.eq_any(&name_ids))
            .load::<Species>(&mut conn)
            .await?;

        let mut species_map: HashMap<Uuid, Species> = HashMap::new();
        for row in rows {
            species_map.insert(row.name_id.clone(), row);
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
        for (name_id, refseq, full, partial) in rows {
            assembly_map.insert(name_id, (refseq as usize, full as usize, partial as usize));
        }


        // enrich the results with the gnl data
        let mut results: Vec<FullTextSearchItem> = Vec::new();
        for result in search_results.iter() {
            match result {
                // the 'maximum resolution' of the search function is at the
                // species level, but it can still match on subspecies so
                // we make sure to enrich the base species as much as possible
                // while still allowing for a quick link to the subspecies
                SearchItem::Species { uuid, score } => {
                    let mut item = TaxonItem::default();
                    item.score = *score;

                    if let Some(species) = species_map.get(&uuid) {
                        item.scientific_name = species.scientific_name.clone();
                        item.scientific_name_authorship = species.species_authority.clone();
                        item.canonical_name = species.canonical_name.clone();
                        item.subspecies = species.subspecies.clone().unwrap_or_default();

                        item.classification = Classification {
                            kingdom: species.kingdom.clone(),
                            phylum: species.phylum.clone(),
                            class: species.class.clone(),
                            order: species.order.clone(),
                            family: species.family.clone(),
                            genus: species.genus.clone(),
                        };
                    }

                    if let Some((refseq, full, partial)) = assembly_map.get(&uuid) {
                        item.assembly_summary = AssemblySummary {
                            reference_genomes: *refseq,
                            whole_genomes: *full,
                            partial_genomes: *partial,
                            barcodes: 0,
                        }
                    }

                    results.push(FullTextSearchItem::Taxon(item));
                }
                // because undescribed species can be numerous and informal we
                // group them together under a genus to make things easier to
                // find in the search
                SearchItem::UndescribedSpecies { genus, score } => {
                    if let Some(undescribed) = undescribed_map.get(genus) {
                        let mut item = GenusItem::default();
                        item.r#type = FullTextType::Genus;
                        item.score = *score;

                        let species = &undescribed[0];
                        // item.scientific_name = format!("{} {}", species.genus.unwrap_or_default(), species.genus_authority.unwrap_or_default());
                        item.scientific_name_authorship = species.genus_authority.clone();
                        item.canonical_name = species.genus.clone();
                        item.undescribed_species = undescribed.iter().map(|s| s.scientific_name.clone()).collect();

                        item.classification = Classification {
                            kingdom: species.kingdom.clone(),
                            phylum: species.phylum.clone(),
                            class: species.class.clone(),
                            order: species.order.clone(),
                            family: species.family.clone(),
                            genus: species.genus.clone(),
                        };

                        // sum up all the assembly stats for every undescribed species
                        for species in undescribed {
                            if let Some((refseq, full, partial)) = assembly_map.get(&species.name_id) {
                                item.assembly_summary.reference_genomes += refseq;
                                item.assembly_summary.whole_genomes += full;
                                item.assembly_summary.partial_genomes += partial;
                            }
                        }

                        results.push(FullTextSearchItem::Genus(item));
                    }
                }

                _ => {}
            };
        }


        // if we are only searching for species shortcut the request
        if data_type == "species" {
            return Ok(FullTextSearchResult {
                records: results
            });
        }


        // get the solr full text search results
        // let mut results = FullTextSearchResult::default();
        // let solr_results = state.solr.full_text(&query).await?;
        // results.records.extend(solr_results.records);

        // // filter out the sequence types that wasn't requested
        // results.records = results.records.into_iter().filter(|record| {
        //     data_type == "all" || data_type == match record {
        //         FullTextSearchItem::Taxon(_) => "species", // should already be filtered out if not 'all'
        //         FullTextSearchItem::Genus(_) => "genus", // should already be filtered out if not 'all'
        //         FullTextSearchItem::GenomeSequence(item) => {
        //             match item.r#type {
        //                 FullTextType::Taxon => "species",
        //                 FullTextType::Genus => "genus",
        //                 FullTextType::ReferenceGenomeSequence => "whole_genomes",
        //                 FullTextType::WholeGenomeSequence => "whole_genomes",
        //                 FullTextType::PartialGenomeSequence => "partial_genomes",
        //                 FullTextType::UnknownGenomeSequence => "unknown_genomes",
        //                 FullTextType::Barcode => "barcodes",
        //             }
        //         },
        //     }
        // }).collect();

        // // mix the results from multiple sources and rank them by the search score
        // results.records.sort_by(|a, b| b.partial_cmp(a).unwrap());

        Ok(FullTextSearchResult::default())
    }
}
