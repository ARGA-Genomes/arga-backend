use arga_core::models::{
    Dataset,
    Name,
    NamePublication,
    NomenclaturalActType,
    Publication,
    Taxon,
    TaxonTreeNode,
    TaxonomicRank,
    ACCEPTED_NAMES,
    SPECIES_RANKS,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Array, Nullable, Text, Varchar};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::extensions::taxa_filters::TaxaFilter;
use super::extensions::Paginate;
use super::models::{Species, TaxonomicStatus};
use super::{schema, schema_gnl, Error, PageResult, PgPool};
use crate::database::extensions::classification_filters::{
    with_classification,
    Classification as ClassificationFilter,
};
use crate::database::extensions::filters::{with_filters, Filter};
use crate::database::extensions::species_filters::{
    // with_parent_classification,
    with_classification as with_species_classification,
};
use crate::database::extensions::sum_if;
use crate::database::extensions::taxa_filters::with_taxa_filters;

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

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = schema::taxon_history)]
pub struct HistoryItem {
    pub source_url: Option<String>,
    pub entity_id: Option<String>,

    #[diesel(embed)]
    pub dataset: Dataset,
    #[diesel(embed)]
    pub taxon: Taxon,
    #[diesel(embed)]
    pub publication: Option<NamePublication>,
}

// because acted_on is an aliased table we don't implement
// Selectable as it will use the `name` table rather than the
// joined aliased table
#[derive(Debug, Queryable)]
#[diesel(table_name = schema::nomenclatural_acts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NomenclaturalAct {
    pub entity_id: String,
    pub act: NomenclaturalActType,
    pub source_url: String,

    pub publication: Publication,
    pub name: Name,
    pub acted_on: Name,
}

#[derive(Debug, Queryable)]
#[diesel(table_name = schema::taxonomic_acts)]
pub struct TaxonomicAct {
    pub entity_id: String,
    pub taxon: Taxon,
    pub accepted_taxon: Option<Taxon>,
    pub source_url: Option<String>,
    pub data_created_at: Option<DateTime<Utc>>,
    pub data_updated_at: Option<DateTime<Utc>>,
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

    pub async fn find(&self, filters: &Vec<TaxaFilter>) -> Result<Vec<Taxon>, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = taxa
            .filter(with_taxa_filters(filters).unwrap())
            .load::<Taxon>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn species(&self, filters: &Vec<Filter>, page: i64, per_page: i64) -> PageResult<Species> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = species
            .filter(status.eq_any(&[
                TaxonomicStatus::Accepted,
                TaxonomicStatus::Undescribed,
                TaxonomicStatus::Hybrid,
            ]))
            .filter(with_filters(&filters).unwrap())
            .order_by(scientific_name)
            .paginate(page)
            .per_page(per_page)
            .load::<(Species, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn ecology_options(&self, _filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
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

    pub async fn ibra_options(&self, _filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
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

    pub async fn imcra_options(&self, _filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
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

    pub async fn state_options(&self, _filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
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

    pub async fn drainage_basin_options(&self, _filters: &Vec<Filter>) -> Result<Vec<String>, Error> {
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

    pub async fn hierarchy(&self, id: &Uuid) -> Result<Vec<TaxonTreeNode>, Error> {
        use schema::taxa;
        use schema_gnl::taxa_dag as dag;

        let mut conn = self.pool.get().await?;

        let nodes = taxa::table
            .inner_join(dag::table.on(dag::taxon_id.eq(taxa::id)))
            .select(dag::all_columns)
            .filter(dag::id.ne(taxa::id))
            .filter(dag::taxon_id.eq(id))
            .load::<TaxonTreeNode>(&mut conn)
            .await?;

        Ok(nodes)
    }

    pub async fn descendant_summary(
        &self,
        classification: &ClassificationFilter,
        rank: TaxonomicRank,
    ) -> Result<Vec<TaxonSummary>, Error> {
        use diesel::dsl::{count_star, sql};
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let rank = rank.to_string().to_lowercase();
        let selector = format!("species.classification->>'{rank}'");
        let rank_group = sql::<Varchar>(&selector);

        let records = species::table
            .filter(species::status.eq_any(ACCEPTED_NAMES))
            .filter(species::rank.eq_any(SPECIES_RANKS))
            .filter(species::classification.retrieve_as_text(rank).is_not_null())
            .filter(with_species_classification(classification))
            .group_by(&rank_group)
            .select((&rank_group, count_star(), sum_if(species::genomes.gt(0)), sum_if(species::total_genomic.gt(0))))
            .load::<TaxonSummary>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn taxon_summary(&self, classification: &ClassificationFilter) -> Result<TaxonSummary, Error> {
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let species = species::table
            .filter(species::status.eq_any(ACCEPTED_NAMES))
            .filter(species::rank.eq_any(SPECIES_RANKS))
            .filter(with_species_classification(classification))
            .count()
            .get_result::<i64>(&mut conn)
            .await?;

        let species_genomes = species::table
            .filter(species::status.eq_any(ACCEPTED_NAMES))
            .filter(species::rank.eq_any(SPECIES_RANKS))
            .filter(with_species_classification(classification))
            .filter(species::genomes.gt(0))
            .count()
            .get_result::<i64>(&mut conn)
            .await?;

        let species_data = species::table
            .filter(species::status.eq_any(ACCEPTED_NAMES))
            .filter(species::rank.eq_any(SPECIES_RANKS))
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
            .select((canonical_name, genomes, loci, specimens, other, total_genomic))
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
            .select((canonical_name, genomes, loci, specimens, other, total_genomic))
            .filter(with_species_classification(filter))
            .order(genomes.desc())
            .limit(10)
            .load::<SpeciesSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn history(&self, taxon_id: &Uuid) -> Result<Vec<HistoryItem>, Error> {
        use schema::{datasets, name_publications as publications, taxa, taxon_history as history, taxon_names};
        let mut conn = self.pool.get().await?;

        let name_ids = taxon_names::table
            .select(taxon_names::name_id)
            .filter(taxon_names::taxon_id.eq(taxon_id))
            .into_boxed();

        let taxon_ids = taxon_names::table
            .select(taxon_names::taxon_id)
            .filter(taxon_names::name_id.eq_any(name_ids))
            .load::<Uuid>(&mut conn)
            .await?;

        let synonym_original_descriptions = history::table
            .filter(history::acted_on.eq_any(taxon_ids.clone()))
            .select(history::taxon_id)
            .into_boxed();

        let items = history::table
            .inner_join(datasets::table)
            .inner_join(taxa::table.on(taxa::id.eq(history::acted_on)))
            .left_join(publications::table)
            .filter(history::taxon_id.eq(taxon_id))
            .or_filter(history::acted_on.eq_any(taxon_ids))
            .or_filter(history::taxon_id.eq_any(synonym_original_descriptions))
            .select(HistoryItem::as_select())
            .order((publications::published_year.asc(), taxa::scientific_name.asc()))
            .load::<HistoryItem>(&mut conn)
            .await?;

        Ok(items)
    }

    pub async fn nomenclatural_acts(&self, taxon_id: &Uuid) -> Result<Vec<NomenclaturalAct>, Error> {
        use schema::{names, nomenclatural_acts as acts, publications, taxon_names, taxonomic_acts};
        let mut conn = self.pool.get().await?;

        // get all the names associated with this taxon in case there are alt names
        let name_ids = taxon_names::table
            .select(taxon_names::name_id)
            .filter(taxon_names::taxon_id.eq(taxon_id))
            .into_boxed();

        // get all the taxa that are linked to the same name since we want acts from all
        // taxonomic systems
        let taxon_ids = taxon_names::table
            .left_join(taxonomic_acts::table.on(taxon_names::taxon_id.eq(taxonomic_acts::taxon_id)))
            .select(taxon_names::taxon_id)
            .filter(taxon_names::name_id.eq_any(name_ids))
            .or_filter(taxonomic_acts::accepted_taxon_id.eq(taxon_id))
            .load::<Uuid>(&mut conn)
            .await?;

        // get any synonyms related to taxa that link to the name since they are part of
        // the name history as well
        let synonym_taxon_ids = taxonomic_acts::table
            .select(taxonomic_acts::taxon_id)
            .filter(taxonomic_acts::taxon_id.eq_any(&taxon_ids))
            .or_filter(taxonomic_acts::accepted_taxon_id.eq_any(&taxon_ids))
            .load::<Uuid>(&mut conn)
            .await?;

        // lastly, we get all the linked names again incase the other taxa or synonyms
        // have alt names linked to them
        let name_ids = taxon_names::table
            .select(taxon_names::name_id)
            .filter(taxon_names::taxon_id.eq_any(taxon_ids))
            .or_filter(taxon_names::taxon_id.eq_any(synonym_taxon_ids))
            .into_boxed();

        let acted_on = diesel::alias!(names as acted_on);

        let items = acts::table
            .inner_join(publications::table)
            .inner_join(names::table.on(names::id.eq(acts::name_id)))
            .inner_join(acted_on.on(acted_on.field(names::id).eq(acts::acted_on_id)))
            .filter(acts::name_id.eq_any(name_ids))
            .select((
                acts::entity_id,
                acts::act,
                acts::source_url,
                Publication::as_select(),
                Name::as_select(),
                acted_on.fields(<Name as Selectable<diesel::pg::Pg>>::construct_selection()),
            ))
            .order((publications::published_year.asc(), names::scientific_name.asc()))
            .load::<NomenclaturalAct>(&mut conn)
            .await?;

        Ok(items)
    }

    pub async fn taxonomic_acts(&self, taxon_id: &Uuid) -> Result<Vec<TaxonomicAct>, Error> {
        use schema::{taxa, taxonomic_acts as acts};
        let mut conn = self.pool.get().await?;

        let accepted = diesel::alias!(taxa as accepted_taxon);

        let items = acts::table
            .inner_join(taxa::table.on(taxa::id.eq(acts::taxon_id)))
            .left_join(accepted.on(acts::accepted_taxon_id.eq(accepted.field(taxa::id).nullable())))
            .filter(acts::taxon_id.eq(taxon_id))
            .or_filter(acts::accepted_taxon_id.eq(taxon_id))
            .select((
                acts::entity_id,
                Taxon::as_select(),
                accepted
                    .fields(<Taxon as Selectable<diesel::pg::Pg>>::construct_selection())
                    .nullable(),
                acts::source_url,
                acts::data_created_at,
                acts::data_updated_at,
            ))
            .order(taxa::scientific_name.asc())
            .load::<TaxonomicAct>(&mut conn)
            .await?;

        Ok(items)
    }
}
