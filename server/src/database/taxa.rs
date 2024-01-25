use arga_core::models::{TaxonTreeNode, Taxon, ACCEPTED_NAMES, TaxonomicRank};
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel::sql_types::{Text, Array, Nullable, Varchar};
use diesel_async::RunQueryDsl;

use crate::database::extensions::filters::{with_filters, Filter, filter_taxa};
use crate::database::extensions::classification_filters::{
    with_classification,
    Classification as ClassificationFilter,
};
use crate::database::extensions::species_filters::{
    // with_parent_classification,
    with_classification as with_species_classification,
    with_filters as with_species_filters,
    Filter as SpeciesFilter,
};
use crate::database::extensions::sum_if;

use super::extensions::Paginate;
use super::{schema, schema_gnl, PgPool, PageResult, Error};
use super::models::{Species, TaxonomicStatus};


sql_function!(fn unnest(x: Nullable<Array<Text>>) -> Text);


#[derive(Debug, Queryable)]
pub struct DataSummary {
    pub canonical_name: String,
    pub genomes: Option<BigDecimal>,
    pub markers: Option<BigDecimal>,
    pub specimens: Option<BigDecimal>,
    pub other: Option<BigDecimal>,
    pub total_genomic: Option<BigDecimal>,
}

#[derive(Debug, Queryable)]
pub struct SpeciesSummary {
    pub name: String,
    pub genomes: i64,
    pub markers: i64,
    pub specimens: i64,
    pub other: i64,
    pub total_genomic: i64,
}

#[derive(Debug, Queryable)]
pub struct TaxonSummary {
    /// The name of the taxon this summary pertains to
    pub canonical_name: String,
    /// Total amount of descendant species
    pub species: i64,
    /// Total amount of descendant species with genomes
    pub species_genomes: i64,
    /// Total amount of descendant species with any genomic data
    pub species_data: i64,
}


#[derive(Clone)]
pub struct TaxaProvider {
    pub pool: PgPool,
}

impl TaxaProvider {
    pub async fn find_by_classification(&self, classification: &ClassificationFilter) -> Result<Taxon, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let taxon = taxa
            .filter(with_classification(classification))
            .first::<Taxon>(&mut conn)
            .await?;

        Ok(taxon)
    }

    pub async fn species(&self, filters: &Vec<Filter>, page: i64, per_page: i64) -> PageResult<Species> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = species
            .filter(taxon_status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .filter(with_filters(&filters).unwrap())
            .order_by(scientific_name)
            .paginate(page)
            .per_page(per_page)
            .load::<(Species, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn ecology_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        // FIXME: reimplement
        Ok(vec![])
        // use schema_gnl::taxa_filter;
        // let mut conn = self.pool.get().await?;

        // let mut options = filter_taxa(filters)
        //     .select(unnest(taxa_filter::ecology))
        //     .distinct()
        //     .load::<String>(&mut conn)
        //     .await?;

        // options.sort();
        // Ok(options)
    }

    pub async fn ibra_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        // FIXME: reimplement
        Ok(vec![])
        // use schema_gnl::taxa_filter;
        // let mut conn = self.pool.get().await?;

        // let mut options = filter_taxa(filters)
        //     .select(unnest(taxa_filter::ibra))
        //     .distinct()
        //     .load::<String>(&mut conn)
        //     .await?;

        // options.sort();
        // Ok(options)
    }

    pub async fn imcra_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        // FIXME: reimplement
        Ok(vec![])
        // use schema_gnl::taxa_filter;
        // let mut conn = self.pool.get().await?;

        // let mut options = filter_taxa(filters)
        //     .select(unnest(taxa_filter::ibra))
        //     .distinct()
        //     .load::<String>(&mut conn)
        //     .await?;

        // options.sort();
        // Ok(options)
    }

    pub async fn state_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        // FIXME: reimplement
        Ok(vec![])
        // use schema_gnl::taxa_filter;
        // let mut conn = self.pool.get().await?;

        // let mut options = filter_taxa(filters)
        //     .select(unnest(taxa_filter::state))
        //     .distinct()
        //     .load::<String>(&mut conn)
        //     .await?;

        // options.sort();
        // Ok(options)
    }

    pub async fn drainage_basin_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        // FIXME: reimplement
        Ok(vec![])
        // use schema_gnl::taxa_filter;
        // let mut conn = self.pool.get().await?;

        // let mut options = filter_taxa(filters)
        //     .select(unnest(taxa_filter::drainage_basin))
        //     .distinct()
        //     .load::<String>(&mut conn)
        //     .await?;

        // options.sort();
        // Ok(options)
    }


    pub async fn hierarchy(&self, classification: &ClassificationFilter) -> Result<Vec<TaxonTreeNode>, Error> {
        use schema::taxa;
        use schema_gnl::taxa_dag as dag;

        let mut conn = self.pool.get().await?;

        // the classification filter helper is typed for the classifications table and will raise
        // compiler errors due to the join against another table/view. rather than making the filters
        // generic we just box the filtered table query first and then join it.
        // we use the same handy technique elsewhere in this file
        let nodes = taxa::table
            .filter(with_classification(classification))
            .into_boxed()
            .inner_join(dag::table.on(dag::taxon_id.eq(taxa::id)))
            .select(dag::all_columns)
            .load::<TaxonTreeNode>(&mut conn)
            .await?;

        Ok(nodes)
    }


    pub async fn descendant_summary(&self, classification: &ClassificationFilter, rank: TaxonomicRank) -> Result<Vec<TaxonSummary>, Error> {
        use diesel::dsl::{count_star, sql};
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let selector = format!("species.classification->>'{}'", rank.to_string().to_lowercase());

        let records = species::table
            .filter(with_species_classification(classification))
            .filter(species::taxon_status.eq_any(ACCEPTED_NAMES))
            .filter(sql::<Varchar>(&selector).is_not_null())
            .group_by(sql::<Varchar>(&selector))
            .select((
                sql::<Varchar>(&selector),
                count_star(),
                sum_if(species::genomes.gt(0)),
                sum_if(species::total_genomic.gt(0)),
            ))
            .load::<TaxonSummary>(&mut conn)
            .await?;

        Ok(records)
    }


    pub async fn taxon_summary(&self, classification: &ClassificationFilter) -> Result<TaxonSummary, Error> {
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let species = species::table
            .filter(with_species_classification(classification))
            .filter(species::taxon_status.eq_any(ACCEPTED_NAMES))
            .count()
            .get_result::<i64>(&mut conn)
            .await?;

        let species_genomes = species::table
            .filter(with_species_classification(classification))
            .filter(species::genomes.gt(0))
            .count()
            .get_result::<i64>(&mut conn)
            .await?;

        let species_data = species::table
            .filter(with_species_classification(classification))
            .filter(species::total_genomic.gt(0))
            .count()
            .get_result::<i64>(&mut conn)
            .await?;

        Ok(TaxonSummary {
            canonical_name: "".to_string(),
            species,
            species_genomes,
            species_data,
        })
    }


    pub async fn species_summary(&self, filter: &ClassificationFilter) -> Result<Vec<SpeciesSummary>, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let summaries = species
            .select((
                canonical_name,
                genomes,
                loci,
                specimens,
                other,
                total_genomic,
            ))
            .filter(with_species_classification(filter))
            .order(total_genomic.desc())
            .limit(10)
            .load::<SpeciesSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn species_genome_summary(&self, filter: &ClassificationFilter) -> Result<Vec<SpeciesSummary>, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let summaries = species
            .select((
                canonical_name,
                genomes,
                loci,
                specimens,
                other,
                total_genomic,
            ))
            .filter(with_species_classification(filter))
            .order(genomes.desc())
            .limit(10)
            .load::<SpeciesSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn data_summary(&self, filter: &ClassificationFilter) -> Result<Vec<DataSummary>, Error> {
        use diesel::dsl::sum;
        use schema::taxa;
        use schema_gnl::species::dsl::*;

        let mut conn = self.pool.get().await?;

        // FIXME: get child taxa totals
        Ok(vec![])
        // let summaries = species
        //     .group_by(canonical_name)
        //     .select((
        //         classification_canonical_name,
        //         sum(markers),
        //         sum(genomes),
        //         sum(specimens),
        //         sum(other),
        //         sum(total_genomic),
        //     ))
        //     .filter(with_parent_classification(classification))
        //     .load::<DataSummary>(&mut conn)
        //     .await?;

        // Ok(summaries)
    }
}
