use arga_core::models::{
    ACCEPTED_NAMES,
    Dataset,
    Name,
    NomenclaturalActType,
    Publication,
    SpecimenOld as Specimen,
    Taxon,
    TaxonTreeNode,
    TaxonWithDataset,
    TaxonomicRank,
};
use bigdecimal::{BigDecimal, Zero};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Array, Nullable, Text};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::extensions::species_filters::{SortDirection, SpeciesSort};
use super::extensions::taxa_filters::TaxaFilter;
use super::extensions::{Paginate, sum_if};
use super::models::Species;
use super::{Error, PageResult, PgPool, schema, schema_gnl};
use crate::database::extensions::classification_filters::{
    Classification as ClassificationFilter,
    with_classification,
};
use crate::database::extensions::filters::{Filter, with_filters};
use crate::database::extensions::species_filters::{
    with_accepted_classification,
    with_classification as with_species_classification,
    with_sorting,
};
use crate::database::extensions::taxa_filters::with_taxa_filters;

sql_function!(fn unnest(x: Nullable<Array<Text>>) -> Text);


#[derive(Debug, Queryable)]
pub struct DataSummary {
    pub scientific_name: String,
    pub canonical_name: String,

    pub loci: Option<BigDecimal>,
    pub genomes: Option<BigDecimal>,
    pub specimens: Option<BigDecimal>,
    pub other: Option<BigDecimal>,
    pub total_genomic: Option<BigDecimal>,

    pub full_genomes: Option<BigDecimal>,
    pub partial_genomes: Option<BigDecimal>,
    pub complete_genomes: Option<BigDecimal>,
    pub assembly_chromosomes: Option<BigDecimal>,
    pub assembly_scaffolds: Option<BigDecimal>,
    pub assembly_contigs: Option<BigDecimal>,

    pub full_genomes_coverage: Option<BigDecimal>,
    pub partial_genomes_coverage: Option<BigDecimal>,
    pub complete_genomes_coverage: Option<BigDecimal>,
    pub assembly_chromosomes_coverage: Option<BigDecimal>,
    pub assembly_scaffolds_coverage: Option<BigDecimal>,
    pub assembly_contigs_coverage: Option<BigDecimal>,
}

#[derive(Debug, Queryable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Summary {
    pub scientific_name: String,
    pub canonical_name: String,
    pub genomes: Option<BigDecimal>,
    pub loci: Option<BigDecimal>,
    pub specimens: Option<BigDecimal>,
    pub other: Option<BigDecimal>,
    pub total_genomic: Option<BigDecimal>,
}

#[derive(Debug, Queryable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SpeciesSummary {
    pub canonical_name: String,
    pub genomes: i64,
    pub loci: i64,
    pub specimens: i64,
    pub other: i64,
    pub total_genomic: i64,
}

#[derive(Debug, Queryable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RankSummary {
    /// Total amount of taxa in the rank
    pub total: i64,
    /// Total amount of taxa in the rank with genomes
    pub genomes: i64,
    /// Total amount of taxa in the rank with any genomic data
    pub genomic_data: i64,
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
#[diesel(table_name = schema::specimens_old)]
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
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Taxon, Error> {
        use schema::taxa;
        let mut conn = self.pool.get().await?;
        let record = taxa::table.filter(taxa::id.eq(id)).get_result(&mut conn).await?;
        Ok(record)
    }

    pub async fn find_one_by_classification(
        &self,
        classification: &ClassificationFilter,
        dataset_id: &Uuid,
    ) -> Result<Taxon, Error> {
        use schema::taxa;
        let mut conn = self.pool.get().await?;

        let record = taxa::table
            .filter(taxa::dataset_id.eq(dataset_id))
            .filter(with_classification(classification))
            .get_result(&mut conn)
            .await?;
        Ok(record)
    }

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

    pub async fn species(
        &self,
        filters: &Vec<Filter>,
        dataset_id: &Uuid,
        page: i64,
        per_page: i64,
        sort: SpeciesSort,
        direction: SortDirection,
    ) -> PageResult<Species> {
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let query = match with_filters(&filters) {
            Some(predicates) => species::table.filter(predicates).into_boxed(),
            None => species::table.into_boxed(),
        };

        let query = query
            .filter(species::dataset_id.eq(dataset_id))
            .filter(species::status.eq_any(ACCEPTED_NAMES))
            .filter(species::rank.eq_any(SPECIES_RANKS));

        let records = with_sorting(query, sort, direction)
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

    /// Summary statistics for a specific rank below the specified taxon
    pub async fn rank_summary(&self, taxon_id: &Uuid, rank: &TaxonomicRank) -> Result<RankSummary, Error> {
        use diesel::dsl::count_star;
        use schema::taxa;
        use schema_gnl::taxa_tree_stats as stats;

        let mut conn = self.pool.get().await?;

        // the taxa_tree_stats view summarises the total amount of data linked to descendants
        // of the supplied taxon. since we want the number of species that have data we instead
        // filter the taxa tree to species level nodes and count how many of them have a record
        // greater than zero. to get multiple values from one query we leverage the sum_if extension.
        let (total, genomes, genomic_data) = stats::table
            .inner_join(taxa::table.on(taxa::id.eq(stats::id)))
            .filter(stats::taxon_id.eq(taxon_id))
            .filter(taxa::rank.eq(rank))
            .select((
                count_star(),
                sum_if(stats::genomes.gt(BigDecimal::zero())),
                sum_if(stats::total_genomic.gt(BigDecimal::zero())),
            ))
            .get_result::<(i64, i64, i64)>(&mut conn)
            .await?;

        Ok(RankSummary {
            total,
            genomes,
            genomic_data,
        })
    }

    // the top 10 species that have the most genomic data
    pub async fn species_genomic_data_summary(&self, taxon_id: &Uuid) -> Result<Vec<Summary>, Error> {
        use schema::taxa;
        use schema_gnl::taxa_tree_stats as stats;
        let mut conn = self.pool.get().await?;

        // the taxa tree stats view aggregates the stats going up, so by searching for
        // all taxa with a rank of species underneath the provided taxon uuid we make sure
        // that any data linked to a subspecies or variety will be included in the stat for
        // the species itself
        let summaries = stats::table
            .inner_join(taxa::table.on(taxa::id.eq(stats::id)))
            .select((
                taxa::scientific_name,
                taxa::canonical_name,
                stats::genomes,
                stats::loci,
                stats::specimens,
                stats::other,
                stats::total_genomic,
            ))
            .filter(stats::taxon_id.eq(taxon_id))
            .filter(taxa::rank.eq(TaxonomicRank::Species))
            .order(stats::total_genomic.desc())
            .limit(10)
            .load::<Summary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    // the top 10 species that have the most genomes
    pub async fn species_genomes_summary(&self, taxon_id: &Uuid) -> Result<Vec<Summary>, Error> {
        use schema::taxa;
        use schema_gnl::taxa_tree_stats as stats;
        let mut conn = self.pool.get().await?;

        // the taxa tree stats view aggregates the stats going up, so by searching for
        // all taxa with a rank of species underneath the provided taxon uuid we make sure
        // that any data linked to a subspecies or variety will be included in the stat for
        // the species itself
        let summaries = stats::table
            .inner_join(taxa::table.on(taxa::id.eq(stats::id)))
            .select((
                taxa::scientific_name,
                taxa::canonical_name,
                stats::genomes,
                stats::loci,
                stats::specimens,
                stats::other,
                stats::total_genomic,
            ))
            .filter(stats::taxon_id.eq(taxon_id))
            .filter(taxa::rank.eq(TaxonomicRank::Species))
            .order(stats::genomes.desc())
            .limit(10)
            .load::<Summary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn taxon_summary(&self, classification: &ClassificationFilter) -> Result<RankSummary, Error> {
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let species_query = species::table
            .filter(with_accepted_classification(classification))
            .into_boxed();
        let species_genomes_query = species::table
            .filter(with_accepted_classification(classification))
            .into_boxed();
        let species_data_query = species::table
            .filter(with_accepted_classification(classification))
            .into_boxed();


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

        Ok(RankSummary {
            total: species,
            genomes: species_genomes,
            genomic_data: species_data,
        })
    }

    pub async fn species_summary(&self, filter: &ClassificationFilter) -> Result<Vec<SpeciesSummary>, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;
        let query = species
            .select((canonical_name, genomes, loci, specimens, other, total_genomic))
            .into_boxed();


        let summaries = query
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
        let query = species
            .select((canonical_name, genomes, loci, specimens, other, total_genomic))
            .into_boxed();


        let summaries = query
            .filter(with_species_classification(filter))
            .order(genomes.desc())
            .limit(10)
            .load::<SpeciesSummary>(&mut conn)
            .await?;

        Ok(summaries)
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
        use schema::{names, specimens_old as specimens};
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
