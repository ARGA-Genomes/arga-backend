use arga_core::models::{
    ACCEPTED_NAMES,
    Dataset,
    Name,
    NamePublication,
    NomenclaturalActType,
    Publication,
    SPECIES_RANKS,
    Specimen,
    Taxon,
    TaxonTreeNode,
    TaxonWithDataset,
    TaxonomicRank,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Array, Nullable, Text, Varchar};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::extensions::species_filters::{NameAttributeFilter, SpeciesFilter};
use super::extensions::taxa_filters::TaxaFilter;
use super::extensions::{Paginate, sum_if};
use super::models::Species;
use super::{Error, PageResult, PgPool, schema, schema_gnl};
use crate::database::extensions::classification_filters::{
    Classification as ClassificationFilter,
    with_classification,
};
use crate::database::extensions::filters::Filter;
use crate::database::extensions::species_filters::{
    with_accepted_classification,
    with_attribute,
    with_classification as with_species_classification,
    with_species_filters,
};
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


#[derive(Debug, Selectable, Queryable)]
#[diesel(table_name = schema::specimens)]
pub struct TypeSpecimen {
    #[diesel(embed)]
    pub specimen: Specimen,
    #[diesel(embed)]
    pub name: Name,
}


#[derive(Clone)]
pub struct TaxaProvider {
    pub pool: PgPool,
}


impl TaxaProvider {
    pub async fn find_by_classification(
        &self,
        classification: &ClassificationFilter,
    ) -> Result<Vec<TaxonWithDataset>, Error> {
        use schema::datasets;
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = taxa
            .filter(with_classification(classification))
            .into_boxed()
            .inner_join(datasets::table.on(dataset_id.eq(datasets::id)))
            .select((Taxon::as_select(), Dataset::as_select()))
            .load::<TaxonWithDataset>(&mut conn)
            .await?;

        Ok(records)
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

    pub async fn species(&self, filters: &Vec<SpeciesFilter>, page: i64, per_page: i64) -> PageResult<Species> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;
        let species_filters = with_species_filters(&filters);

        let mut query = species
            .filter(status.eq_any(ACCEPTED_NAMES))
            .filter(rank.eq_any(SPECIES_RANKS))
            .into_boxed();

        if let Some(filter) = species_filters {
            query = query.filter(filter);
        }

        let records = query
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
            .filter(with_accepted_classification(classification))
            .filter(species::classification.retrieve_as_text(rank).is_not_null())
            .group_by(&rank_group)
            .select((&rank_group, count_star(), sum_if(species::genomes.gt(0)), sum_if(species::total_genomic.gt(0))))
            .load::<TaxonSummary>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn taxon_summary(
        &self,
        classification: &ClassificationFilter,
        attribute: &Option<NameAttributeFilter>,
    ) -> Result<TaxonSummary, Error> {
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let mut species_query = species::table
            .filter(with_accepted_classification(classification))
            .into_boxed();
        let mut species_genomes_query = species::table
            .filter(with_accepted_classification(classification))
            .into_boxed();
        let mut species_data_query = species::table
            .filter(with_accepted_classification(classification))
            .into_boxed();

        match attribute {
            Some(attr) => {
                species_query = species_query.filter(with_attribute(attr));
                species_genomes_query = species_genomes_query.filter(with_attribute(attr));
                species_data_query = species_data_query.filter(with_attribute(attr));
            }
            None => {}
        }

        let species = species_query.count().get_result::<i64>(&mut conn).await?;

        let species_genomes = species_genomes_query
            .filter(species::genomes.gt(0))
            .count()
            .get_result::<i64>(&mut conn)
            .await?;

        let species_data = species_data_query
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

    pub async fn species_summary(
        &self,
        filter: &ClassificationFilter,
        attribute: &Option<NameAttributeFilter>,
    ) -> Result<Vec<SpeciesSummary>, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;
        let query = species
            .select((canonical_name, genomes, loci, specimens, other, total_genomic))
            .into_boxed();

        let query = match attribute {
            Some(attr) => query.filter(with_attribute(attr)),
            None => query,
        };

        let summaries = query
            .filter(with_species_classification(filter))
            .order(total_genomic.desc())
            .limit(10)
            .load::<SpeciesSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn species_genome_summary(
        &self,
        filter: &ClassificationFilter,
        attribute: &Option<NameAttributeFilter>,
    ) -> Result<Vec<SpeciesSummary>, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;
        let query = species
            .select((canonical_name, genomes, loci, specimens, other, total_genomic))
            .into_boxed();

        let query = match attribute {
            Some(attr) => query.filter(with_attribute(attr)),
            None => query,
        };

        let summaries = query
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

    pub async fn type_specimens(&self, taxon_id: &Uuid) -> Result<Vec<TypeSpecimen>, Error> {
        use schema::{names, specimens};
        let mut conn = self.pool.get().await?;

        let name_ids = self.all_associated_names(taxon_id).await?;

        let specimens = specimens::table
            .inner_join(names::table.on(names::id.eq(specimens::name_id)))
            .select((Specimen::as_select(), Name::as_select()))
            .filter(specimens::name_id.eq_any(name_ids))
            .filter(specimens::type_status.is_not_null())
            .limit(10)
            .load::<TypeSpecimen>(&mut conn)
            .await?;

        Ok(specimens)
    }

    pub async fn all_associated_names(&self, taxon_id: &Uuid) -> Result<Vec<Uuid>, Error> {
        use schema::{taxon_names, taxonomic_acts};
        let mut conn = self.pool.get().await?;

        // get all the names associated with this taxon in case there are alt names
        let name_ids = taxon_names::table
            .select(taxon_names::name_id)
            .filter(taxon_names::taxon_id.eq(taxon_id))
            .into_boxed();

        // get all the taxa that are linked to the same name since we want names from all
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
            .load::<Uuid>(&mut conn)
            .await?;

        Ok(name_ids)
    }
}
