use std::collections::HashMap;
use std::path::PathBuf;

use arga_core::crdt::lww::Map;
use arga_core::crdt::{Frame, Version};
use arga_core::models::{
    Action,
    DatasetVersion,
    TaxonAtom,
    TaxonOperation,
    TaxonomicActAtom,
    TaxonomicActOperation,
    TaxonomicActType,
    TaxonomicRank,
    TaxonomicStatus,
};
use arga_core::schema;
use bigdecimal::BigDecimal;
use diesel::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;
use xxhash_rust::xxh3::Xxh3;

use crate::data::oplogger::get_pool;
use crate::data::{Error, ParseError};


fn taxonomic_rank_from_str<'de, D>(deserializer: D) -> Result<TaxonomicRank, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    str_to_taxonomic_rank(&s).map_err(serde::de::Error::custom)
}

fn taxonomic_status_from_str<'de, D>(deserializer: D) -> Result<TaxonomicStatus, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    str_to_taxonomic_status(&s).map_err(serde::de::Error::custom)
}


#[derive(Debug, Clone, Deserialize)]
struct Record {
    taxon_id: String,
    // accepted_name_usage_id: Option<String>,
    // parent_name_usage_id: Option<String>,
    parent_taxon: Option<String>,

    scientific_name: String,
    scientific_name_authorship: Option<String>,
    canonical_name: String,
    accepted_usage_taxon: Option<String>,
    // accepted_name_usage: Option<String>,
    // parent_name_usage: Option<String>,
    #[serde(deserialize_with = "taxonomic_rank_from_str")]
    taxon_rank: TaxonomicRank,
    #[serde(deserialize_with = "taxonomic_status_from_str")]
    taxonomic_status: TaxonomicStatus,
    nomenclatural_code: String,
    // nomenclatural_status: Option<String>,
    // name_published_in: Option<String>,
    // name_published_in_year: Option<String>,
    // name_published_in_url: Option<String>,
    citation: Option<String>,
    references: Option<String>,
    last_updated: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Taxon {
    entity_id: String,
    taxon_id: String,
    parent_taxon: Option<String>,

    scientific_name: String,
    scientific_name_authorship: Option<String>,
    canonical_name: String,
    nomenclatural_code: String,

    taxon_rank: TaxonomicRank,
    taxonomic_status: TaxonomicStatus,

    citation: Option<String>,
    references: Option<String>,
    // description: Option<String>,
    // remarks: Option<String>,
    last_updated: Option<String>,
}

impl From<Map<TaxonAtom>> for Taxon {
    fn from(value: Map<TaxonAtom>) -> Self {
        use TaxonAtom::*;

        let mut taxon = Taxon {
            entity_id: value.entity_id,
            ..Default::default()
        };

        for val in value.atoms.into_values() {
            match val {
                Empty => {}
                TaxonId(value) => taxon.taxon_id = value,
                ParentTaxon(value) => taxon.parent_taxon = Some(value),
                ScientificName(value) => taxon.scientific_name = value,
                Authorship(value) => taxon.scientific_name_authorship = Some(value),
                CanonicalName(value) => taxon.canonical_name = value,
                NomenclaturalCode(value) => taxon.nomenclatural_code = value,
                TaxonomicRank(value) => taxon.taxon_rank = value,
                TaxonomicStatus(value) => taxon.taxonomic_status = value,
                Citation(value) => taxon.citation = Some(value),
                References(value) => taxon.references = Some(value),
                LastUpdated(value) => taxon.last_updated = Some(value),

                // fields currently not supported
                AcceptedNameUsageId(_value) => {}
                ParentNameUsageId(_value) => {}
                AcceptedNameUsage(_value) => {}
                ParentNameUsage(_value) => {}
                NomenclaturalStatus(_value) => {}
                NamePublishedIn(_value) => {}
                NamePublishedInYear(_value) => {}
                NamePublishedInUrl(_value) => {}
            }
        }

        taxon
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct TaxonomicAct {
    entity_id: String,
    taxon: String,
    accepted_taxon: Option<String>,
    act: Option<TaxonomicActType>,
    publication: Option<String>,
    publication_date: Option<String>,
    source_url: Option<String>,
}

impl From<Map<TaxonomicActAtom>> for TaxonomicAct {
    fn from(value: Map<TaxonomicActAtom>) -> Self {
        use TaxonomicActAtom::*;

        let mut act = TaxonomicAct {
            entity_id: value.entity_id,
            ..Default::default()
        };

        for val in value.atoms.into_values() {
            match val {
                Empty => {}
                Publication(value) => act.publication = Some(value),
                PublicationDate(value) => act.publication_date = Some(value),
                Taxon(value) => act.taxon = value,
                AcceptedTaxon(value) => act.accepted_taxon = Some(value),
                Act(value) => act.act = Some(value),
                SourceUrl(value) => act.source_url = Some(value),
            }
        }

        act
    }
}


pub struct Taxa {
    pub path: PathBuf,
    pub dataset_version_id: Uuid,
}

impl Taxa {
    pub fn taxa(&self) -> Result<Vec<TaxonOperation>, Error> {
        use TaxonAtom::*;

        let mut records: Vec<Record> = Vec::new();
        for row in csv::Reader::from_path(&self.path)?.deserialize() {
            records.push(row?);
        }

        let mut last_version = Version::new();
        let mut operations: Vec<TaxonOperation> = Vec::new();

        for record in records.into_iter() {
            // because arga supports multiple taxonomic systems we use the taxon_id
            // from the system as the unique entity id. if we used the scientific name
            // instead then we would combine and reduce changes from all systems which
            // is not desireable for our purposes
            let mut hasher = Xxh3::new();
            hasher.update(record.taxon_id.as_bytes());
            let hash = hasher.digest();

            let mut frame = TaxonFrame::create(self.dataset_version_id, hash.to_string(), last_version);
            frame.push(TaxonId(record.taxon_id));
            frame.push(ScientificName(record.scientific_name));
            frame.push(CanonicalName(record.canonical_name));
            frame.push(TaxonomicRank(record.taxon_rank));
            frame.push(TaxonomicStatus(record.taxonomic_status));
            frame.push(NomenclaturalCode(record.nomenclatural_code));

            if let Some(value) = record.parent_taxon {
                frame.push(ParentTaxon(value));
            }
            // if let Some(value) = record.accepted_name_usage_id {
            //     frame.push(AcceptedNameUsageId(value));
            // }
            // if let Some(value) = record.parent_name_usage_id {
            //     frame.push(ParentNameUsageId(value));
            // }
            // if let Some(value) = record.accepted_name_usage {
            //     frame.push(AcceptedNameUsage(value));
            // }
            // if let Some(value) = record.parent_name_usage {
            //     frame.push(ParentNameUsage(value));
            // }
            if let Some(value) = record.scientific_name_authorship {
                frame.push(Authorship(value));
            }
            // if let Some(value) = record.nomenclatural_status {
            //     frame.push(NomenclaturalStatus(value));
            // }
            // if let Some(value) = record.name_published_in {
            //     frame.push(NamePublishedIn(value));
            // }
            // if let Some(value) = record.name_published_in_year {
            //     frame.push(NamePublishedInYear(value));
            // }
            // if let Some(value) = record.name_published_in_url {
            //     frame.push(NamePublishedInUrl(value));
            // }
            if let Some(value) = record.citation {
                frame.push(Citation(value));
            }
            if let Some(value) = record.references {
                frame.push(References(value));
            }
            if let Some(value) = record.last_updated {
                frame.push(LastUpdated(value));
            }

            last_version = frame.frame.current;
            operations.extend(frame.frame.operations);
        }

        Ok(operations)
    }

    pub fn acts(&self) -> Result<Vec<TaxonomicActOperation>, Error> {
        use TaxonomicActAtom::*;

        let mut records: Vec<Record> = Vec::new();
        for row in csv::Reader::from_path(&self.path)?.deserialize() {
            records.push(row?);
        }

        let mut last_version = Version::new();
        let mut operations: Vec<TaxonomicActOperation> = Vec::new();

        for record in records.into_iter() {
            let act = match record.taxonomic_status {
                TaxonomicStatus::Synonym => Some(TaxonomicActType::Synonym),
                TaxonomicStatus::Homonym => Some(TaxonomicActType::Homonym),
                TaxonomicStatus::Unaccepted => Some(TaxonomicActType::Unaccepted),
                TaxonomicStatus::NomenclaturalSynonym => Some(TaxonomicActType::NomenclaturalSynonym),
                TaxonomicStatus::TaxonomicSynonym => Some(TaxonomicActType::TaxonomicSynonym),
                TaxonomicStatus::ReplacedSynonym => Some(TaxonomicActType::ReplacedSynonym),
                _ => None,
            };

            // skip anything that isn't a synonym
            if act.is_none() {
                continue;
            }

            // because arga supports multiple taxonomic systems we use the taxon_id
            // from the system as the unique entity id. if we used the scientific name
            // instead then we would combine and reduce changes from all systems which
            // is not desireable for our purposes
            let mut hasher = Xxh3::new();
            hasher.update(record.taxon_id.as_bytes());
            let hash = hasher.digest();

            let mut frame = TaxonomicActFrame::create(self.dataset_version_id, hash.to_string(), last_version);
            frame.push(Taxon(record.scientific_name));

            if let Some(value) = act {
                frame.push(Act(value));
            }
            if let Some(value) = record.accepted_usage_taxon {
                frame.push(AcceptedTaxon(value));
            }
            // if let Some(value) = record.name_published_in {
            //     frame.push(Publication(value));
            // }
            // if let Some(value) = record.name_published_in_year {
            //     frame.push(PublicationDate(value));
            // }
            if let Some(value) = record.references {
                frame.push(SourceUrl(value));
            }

            last_version = frame.frame.current;
            operations.extend(frame.frame.operations);
        }

        Ok(operations)
    }
}


pub fn process(path: PathBuf, dataset_version: DatasetVersion) -> Result<(), Error> {
    use schema::{taxa_logs, taxonomic_act_logs};

    let taxa = Taxa {
        path: path.clone(),
        dataset_version_id: dataset_version.id,
    };

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    info!("Loading taxon operations");
    let taxon_ops = taxa_logs::table
        .order(taxa_logs::operation_id.asc())
        .load::<TaxonOperation>(&mut conn)?;

    info!("Reducing taxon operations");
    let records = taxa.taxa()?;
    let reduced = reduce_operations(taxon_ops, records)?;

    info!("Importing taxon operations");
    import_taxa(reduced)?;

    info!("Loading taxonomic act operations");
    let taxon_act_ops = taxonomic_act_logs::table
        .order(taxonomic_act_logs::operation_id.asc())
        .load::<TaxonomicActOperation>(&mut conn)?;

    info!("Reducing taxonomic act operations");
    let records = taxa.acts()?;
    let reduced = reduce_operations(taxon_act_ops, records)?;

    info!("Importing taxonomic act operations");
    import_acts(reduced)?;

    Ok(())
}

fn import_taxa(records: Vec<TaxonOperation>) -> Result<(), Error> {
    use schema::taxa_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    for chunk in records.chunks(1000) {
        diesel::insert_into(taxa_logs)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
    }

    Ok(())
}

fn import_acts(records: Vec<TaxonomicActOperation>) -> Result<(), Error> {
    use schema::taxonomic_act_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    for chunk in records.chunks(1000) {
        diesel::insert_into(taxonomic_act_logs)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
    }

    Ok(())
}

fn merge_ops<T, A>(existing_ops: Vec<T>, new_ops: Vec<T>) -> Result<HashMap<String, Vec<T>>, Error>
where
    T: arga_core::models::LogOperation<A>,
{
    let mut grouped: HashMap<String, Vec<T>> = HashMap::new();

    for op in existing_ops.into_iter() {
        grouped.entry(op.id().clone()).or_default().push(op);
    }

    for op in new_ops.into_iter() {
        grouped.entry(op.id().clone()).or_default().push(op);
    }

    Ok(grouped)
}

fn reduce_operations<T, A>(existing_ops: Vec<T>, new_ops: Vec<T>) -> Result<Vec<T>, Error>
where
    A: ToString + Clone + PartialEq,
    T: arga_core::models::LogOperation<A> + Clone,
{
    let entities = merge_ops(existing_ops, new_ops)?;
    let mut merged = Vec::new();

    for (key, ops) in entities.into_iter() {
        let mut map = Map::new(key);
        let reduced = map.reduce(&ops);
        merged.extend(reduced);
    }

    Ok(merged)
}


pub fn reduce() -> Result<(), Error> {
    use schema::taxa_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    let ops = taxa_logs.order(operation_id.asc()).load::<TaxonOperation>(&mut conn)?;

    let entities = merge_ops(ops, vec![])?;
    let mut taxa = Vec::new();

    for (key, ops) in entities.into_iter() {
        let mut map = Map::new(key);
        map.reduce(&ops);
        taxa.push(Taxon::from(map));
    }

    let mut writer = csv::Writer::from_writer(std::io::stdout());

    for taxon in taxa {
        writer.serialize(taxon)?;
    }

    Ok(())
}

pub fn reduce_acts() -> Result<(), Error> {
    use schema::taxonomic_act_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    let ops = taxonomic_act_logs
        .order(operation_id.asc())
        .load::<TaxonomicActOperation>(&mut conn)?;

    let entities = merge_ops(ops, vec![])?;
    let mut taxa = Vec::new();

    for (key, ops) in entities.into_iter() {
        let mut map = Map::new(key);
        map.reduce(&ops);
        taxa.push(TaxonomicAct::from(map));
    }

    let mut writer = csv::Writer::from_writer(std::io::stdout());

    for taxon in taxa {
        writer.serialize(taxon)?;
    }

    Ok(())
}


fn str_to_taxonomic_rank(value: &str) -> Result<TaxonomicRank, ParseError> {
    use TaxonomicRank::*;

    match value.to_lowercase().as_str() {
        "domain" => Ok(Domain),
        "superkingdom" => Ok(Superkingdom),
        "kingdom" => Ok(Kingdom),
        "subkingdom" => Ok(Subkingdom),
        "infrakingdom" => Ok(Infrakingdom),
        "superphylum" => Ok(Superphylum),
        "phylum" => Ok(Phylum),
        "subphylum" => Ok(Subphylum),
        "infraphylum" => Ok(Infraphylum),
        "parvphylum" => Ok(Parvphylum),
        "gigaclass" => Ok(Gigaclass),
        "megaclass" => Ok(Megaclass),
        "superclass" => Ok(Superclass),
        "class" => Ok(Class),
        "subclass" => Ok(Subclass),
        "infraclass" => Ok(Infraclass),
        "subterclass" => Ok(Subterclass),
        "superorder" => Ok(Superorder),
        "order" => Ok(Order),
        "hyporder" => Ok(Hyporder),
        "minorder" => Ok(Minorder),
        "suborder" => Ok(Suborder),
        "infraorder" => Ok(Infraorder),
        "parvorder" => Ok(Parvorder),
        "epifamily" => Ok(Epifamily),
        "superfamily" => Ok(Superfamily),
        "family" => Ok(Family),
        "subfamily" => Ok(Subfamily),
        "supertribe" => Ok(Supertribe),
        "tribe" => Ok(Tribe),
        "subtribe" => Ok(Subtribe),
        "genus" => Ok(Genus),
        "subgenus" => Ok(Subgenus),
        "species" => Ok(Species),
        "subspecies" => Ok(Subspecies),
        "variety" => Ok(Variety),
        "subvariety" => Ok(Subvariety),
        "natio" => Ok(Natio),
        "mutatio" => Ok(Mutatio),
        "unranked" => Ok(Unranked),
        "higher taxon" => Ok(HigherTaxon),
        "aggregate genera" => Ok(AggregateGenera),
        "aggregate species" => Ok(AggregateSpecies),
        "cohort" => Ok(Cohort),
        "subcohort" => Ok(Subcohort),
        "division" => Ok(Division),
        "phylum (division)" => Ok(Division),
        "incertae sedis" => Ok(IncertaeSedis),
        "infragenus" => Ok(Infragenus),
        "section" => Ok(Section),
        "subsection" => Ok(Subsection),
        "subdivision" => Ok(Subdivision),
        "subphylum (subdivision)" => Ok(Subdivision),

        "regnum" => Ok(Regnum),
        "familia" => Ok(Familia),
        "classis" => Ok(Classis),
        "ordo" => Ok(Ordo),
        "varietas" => Ok(Varietas),
        "forma" => Ok(Forma),
        "subforma" => Ok(Subforma),
        "subclassis" => Ok(Subclassis),
        "superordo" => Ok(Superordo),
        "sectio" => Ok(Sectio),
        "subsectio" => Ok(Subsectio),
        "nothovarietas" => Ok(Nothovarietas),
        "subvarietas" => Ok(Subvarietas),
        "series" => Ok(Series),
        "subseries" => Ok(Subseries),
        "superspecies" => Ok(Superspecies),
        "infraspecies" => Ok(Infraspecies),
        "subfamilia" => Ok(Subfamilia),
        "subordo" => Ok(Subordo),
        "regio" => Ok(Regio),
        "special form" => Ok(SpecialForm),

        "" => Ok(Unranked),

        val => Err(ParseError::InvalidValue(val.to_string())),
    }
}


pub fn str_to_taxonomic_status(value: &str) -> Result<TaxonomicStatus, ParseError> {
    use TaxonomicStatus::*;

    match value.to_lowercase().as_str() {
        "valid" => Ok(Accepted),
        "valid name" => Ok(Accepted),
        "accepted" => Ok(Accepted),
        "accepted name" => Ok(Accepted),

        "undescribed" => Ok(Undescribed),
        "species inquirenda" => Ok(SpeciesInquirenda),
        "taxon inquirendum" => Ok(TaxonInquirendum),
        "manuscript name" => Ok(ManuscriptName),
        "hybrid" => Ok(Hybrid),

        "unassessed" => Ok(Unassessed),
        "unavailable name" => Ok(Unavailable),
        "uncertain" => Ok(Uncertain),
        "unjustified emendation" => Ok(UnjustifiedEmendation),

        "synonym" => Ok(Synonym),
        "junior synonym" => Ok(Synonym),
        "junior objective synonym" => Ok(Synonym),
        "junior subjective synonym" => Ok(Synonym),
        "later synonym" => Ok(Synonym),

        "homonym" => Ok(Homonym),
        "junior homonym" => Ok(Homonym),
        "unreplaced junior homonym" => Ok(Homonym),

        "invalid" => Ok(Unaccepted),
        "invalid name" => Ok(Unaccepted),
        "unaccepted" => Ok(Unaccepted),
        "unaccepted name" => Ok(Unaccepted),
        // "excluded" => Ok(Unaccepted),
        "informal" => Ok(Informal),
        "informal name" => Ok(Informal),

        "placeholder" => Ok(Placeholder),
        "temporary name" => Ok(Placeholder),

        "basionym" => Ok(Basionym),
        "nomenclatural synonym" => Ok(NomenclaturalSynonym),
        "taxonomic synonym" => Ok(TaxonomicSynonym),
        "replaced synonym" => Ok(ReplacedSynonym),

        "incorrect original spelling" => Ok(Misspelled),
        "misspelling" => Ok(Misspelled),

        "orthographic variant" => Ok(OrthographicVariant),
        "excluded" => Ok(Excluded),

        "misapplied" => Ok(Misapplied),
        "misapplication" => Ok(Misapplied),
        "alternative name" => Ok(AlternativeName),
        "alternative representation" => Ok(AlternativeName),

        "pro parte misapplied" => Ok(ProParteMisapplied),
        "pro parte taxonomic synonym" => Ok(ProParteTaxonomicSynonym),

        "doubtful misapplied" => Ok(DoubtfulMisapplied),
        "doubtful taxonomic synonym" => Ok(DoubtfulTaxonomicSynonym),
        "doubtful pro parte misapplied" => Ok(DoubtfulProParteMisapplied),
        "doubtful pro parte taxonomic synonym" => Ok(DoubtfulProParteTaxonomicSynonym),

        "nomen dubium" => Ok(NomenDubium),
        "nomen nudum" => Ok(NomenNudum),
        "nomen oblitum" => Ok(NomenOblitum),

        "interim unpublished" => Ok(InterimUnpublished),
        "superseded combination" => Ok(SupersededCombination),
        "superseded rank" => Ok(SupersededRank),
        "incorrect grammatical agreement of specific epithet" => Ok(IncorrectGrammaticalAgreementOfSpecificEpithet),

        val => Err(ParseError::InvalidValue(val.to_string())),
    }
}


pub struct TaxonFrame {
    dataset_version_id: Uuid,
    entity_id: String,
    frame: Frame<TaxonOperation>,
}

impl TaxonFrame {
    pub fn create(dataset_version_id: Uuid, entity_id: String, last_version: Version) -> TaxonFrame {
        let mut frame = Frame::new(last_version);

        frame.push(TaxonOperation {
            operation_id: frame.next.into(),
            parent_id: frame.current.into(),
            dataset_version_id,
            entity_id: entity_id.clone(),
            action: Action::Create,
            atom: TaxonAtom::Empty,
        });

        TaxonFrame {
            dataset_version_id,
            entity_id,
            frame,
        }
    }

    pub fn push(&mut self, atom: TaxonAtom) {
        let operation_id: BigDecimal = self.frame.next.into();
        let parent_id = self
            .frame
            .operations
            .last()
            .map(|op| op.operation_id.clone())
            .unwrap_or(operation_id.clone());

        let op = TaxonOperation {
            operation_id,
            parent_id,
            dataset_version_id: self.dataset_version_id,
            entity_id: self.entity_id.clone(),
            action: Action::Update,
            atom,
        };

        self.frame.push(op);
    }

    pub fn operations(&self) -> &Vec<TaxonOperation> {
        &self.frame.operations
    }
}

pub struct TaxonomicActFrame {
    dataset_version_id: Uuid,
    entity_id: String,
    frame: Frame<TaxonomicActOperation>,
}

impl TaxonomicActFrame {
    pub fn create(dataset_version_id: Uuid, entity_id: String, last_version: Version) -> TaxonomicActFrame {
        let mut frame = Frame::new(last_version);

        frame.push(TaxonomicActOperation {
            operation_id: frame.next.into(),
            parent_id: frame.current.into(),
            dataset_version_id,
            entity_id: entity_id.clone(),
            action: Action::Create,
            atom: TaxonomicActAtom::Empty,
        });

        TaxonomicActFrame {
            dataset_version_id,
            entity_id,
            frame,
        }
    }

    pub fn push(&mut self, atom: TaxonomicActAtom) {
        let operation_id: BigDecimal = self.frame.next.into();
        let parent_id = self
            .frame
            .operations
            .last()
            .map(|op| op.operation_id.clone())
            .unwrap_or(operation_id.clone());

        let op = TaxonomicActOperation {
            operation_id,
            parent_id,
            dataset_version_id: self.dataset_version_id,
            entity_id: self.entity_id.clone(),
            action: Action::Update,
            atom,
        };

        self.frame.push(op);
    }

    pub fn operations(&self) -> &Vec<TaxonomicActOperation> {
        &self.frame.operations
    }
}
