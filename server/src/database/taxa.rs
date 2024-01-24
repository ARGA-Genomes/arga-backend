use arga_core::models::{TaxonTreeNode, Taxon};
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel::sql_types::{Text, Array, Nullable};
use diesel_async::RunQueryDsl;

use crate::database::extensions::filters::{with_filters, Filter, filter_taxa};
use crate::database::extensions::classification_filters::{
    with_classification,
    Classification as ClassificationFilter,
};
use crate::database::extensions::species_filters::{
    with_parent_classification,
    with_classification as with_species_classification,
};
use crate::database::extensions::sum_if;

use super::extensions::Paginate;
use super::{schema, schema_gnl, PgPool, PageResult, Error};
use super::models::{FilteredTaxon, TaxonomicStatus};


sql_function!(fn unnest(x: Nullable<Array<Text>>) -> Text);


#[derive(Debug, Queryable)]
pub struct DataSummary {
    pub canonical_name: String,
    pub markers: Option<BigDecimal>,
    pub genomes: Option<BigDecimal>,
    pub specimens: Option<BigDecimal>,
    pub other: Option<BigDecimal>,
    pub total_genomic: Option<BigDecimal>,
}

#[derive(Debug, Queryable)]
pub struct SpeciesSummary {
    pub name: String,
    pub markers: i64,
    pub genomes: i64,
    pub specimens: i64,
    pub other: i64,
    pub total_genomic: i64,
}

#[derive(Debug, Queryable)]
pub struct TaxonSummary {
    /// Total amount of child taxa
    pub children: i64,
    /// Total amount of child taxa that have species with genomes
    pub children_genomes: i64,
    /// Total amount of child taxa that have species with any genomic data
    pub children_data: i64,

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

    pub async fn species(&self, filters: &Vec<Filter>, page: i64, per_page: i64) -> PageResult<FilteredTaxon> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let species = taxa_filter::table
            .select(taxa_filter::all_columns)
            .filter(taxa_filter::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
            .filter(with_filters(&filters).unwrap())
            .order_by(taxa_filter::scientific_name)
            .paginate(page)
            .per_page(per_page)
            .load::<(FilteredTaxon, i64)>(&mut conn)
            .await?;

        Ok(species.into())
    }

    pub async fn ecology_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let mut options = filter_taxa(filters)
            .select(unnest(taxa_filter::ecology))
            .distinct()
            .load::<String>(&mut conn)
            .await?;

        options.sort();
        Ok(options)
    }

    pub async fn ibra_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let mut options = filter_taxa(filters)
            .select(unnest(taxa_filter::ibra))
            .distinct()
            .load::<String>(&mut conn)
            .await?;

        options.sort();
        Ok(options)
    }

    pub async fn imcra_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let mut options = filter_taxa(filters)
            .select(unnest(taxa_filter::ibra))
            .distinct()
            .load::<String>(&mut conn)
            .await?;

        options.sort();
        Ok(options)
    }

    pub async fn state_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let mut options = filter_taxa(filters)
            .select(unnest(taxa_filter::state))
            .distinct()
            .load::<String>(&mut conn)
            .await?;

        options.sort();
        Ok(options)
    }

    pub async fn drainage_basin_options(&self, filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
        use schema_gnl::taxa_filter;
        let mut conn = self.pool.get().await?;

        let mut options = filter_taxa(filters)
            .select(unnest(taxa_filter::drainage_basin))
            .distinct()
            .load::<String>(&mut conn)
            .await?;

        options.sort();
        Ok(options)
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


    pub async fn taxon_summary(&self, classification: &ClassificationFilter) -> Result<TaxonSummary, Error> {
        use schema::taxa;
        use schema_gnl::{taxa_dag as dag, classification_species as species};

        let mut conn = self.pool.get().await?;

        let species = taxa::table
            .filter(with_classification(classification))
            .into_boxed()
            .inner_join(dag::table.on(dag::id.eq(taxa::id)))
            .filter(taxa::status.eq_any(&[
                TaxonomicStatus::Accepted,
                TaxonomicStatus::Undescribed,
                TaxonomicStatus::Hybrid,
            ]))
            .count()
            .get_result::<i64>(&mut conn)
            .await?;

        // get the total amount of child taxa. we don't need the dag for this
        let child = diesel::alias!(taxa as children);
        let children = taxa::table
            .filter(with_classification(classification))
            .into_boxed()
            .inner_join(child.on(child.field(taxa::parent_id).eq(taxa::id.nullable())))
            .count()
            .get_result::<i64>(&mut conn)
            .await?;

        // get the total amount of species with genomes and genomic data
        let (species_genomes, species_data) = species::table
            .group_by(species::classification_canonical_name)
            .select((sum_if(species::genomes.gt(0)), sum_if(species::total_genomic.gt(0))))
            .filter(with_species_classification(classification))
            .get_result::<(i64, i64)>(&mut conn)
            .await
            .optional()?
            .unwrap_or_default();

        // get the total amount of child taxa with genomes and genomic data
        let (children_genomes, children_data) = species::table
            .group_by(species::classification_canonical_name)
            .select((sum_if(species::genomes.gt(0)), sum_if(species::total_genomic.gt(0))))
            .filter(with_parent_classification(classification))
            .get_result::<(i64, i64)>(&mut conn)
            .await
            .optional()?
            .unwrap_or_default();

        Ok(TaxonSummary {
            children,
            children_genomes,
            children_data,
            species,
            species_genomes,
            species_data,
        })
    }


    pub async fn species_summary(&self, classification: &ClassificationFilter) -> Result<Vec<SpeciesSummary>, Error> {
        use schema_gnl::classification_species::dsl::*;
        let mut conn = self.pool.get().await?;

        let summaries = classification_species
            .select((
                canonical_name,
                markers,
                genomes,
                specimens,
                other,
                total_genomic,
            ))
            .filter(with_species_classification(classification))
            .order(total_genomic.desc())
            .limit(10)
            .load::<SpeciesSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn species_genome_summary(&self, classification: &ClassificationFilter) -> Result<Vec<SpeciesSummary>, Error> {
        use schema_gnl::classification_species::dsl::*;
        let mut conn = self.pool.get().await?;

        let summaries = classification_species
            .select((
                canonical_name,
                markers,
                genomes,
                specimens,
                other,
                total_genomic,
            ))
            .filter(with_species_classification(classification))
            .order(genomes.desc())
            .limit(10)
            .load::<SpeciesSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn data_summary(&self, classification: &ClassificationFilter) -> Result<Vec<DataSummary>, Error> {
        use diesel::dsl::sum;
        use schema_gnl::classification_species::dsl::*;

        let mut conn = self.pool.get().await?;

        let summaries = classification_species
            .group_by(classification_canonical_name)
            .select((
                classification_canonical_name,
                sum(markers),
                sum(genomes),
                sum(specimens),
                sum(other),
                sum(total_genomic),
            ))
            .filter(with_parent_classification(classification))
            .load::<DataSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }
}
