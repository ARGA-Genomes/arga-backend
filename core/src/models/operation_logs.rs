use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use core_derive::{Atom, OperationLog};
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Jsonb;
use diesel::{AsExpression, Associations, FromSqlRow, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use strum::Display;
use uuid::Uuid;

use super::logs::{Action, LogOperation, LogOperationDataset};
use super::{schema, Dataset, DatasetVersion, PublicationType, TaxonomicRank, TaxonomicStatus};
use crate::crdt::DataFrameOperation;
use crate::models::NomenclaturalActType;


#[derive(Atom, Debug, Default, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum TaxonAtom {
    #[default]
    Empty,
    EntityId(String),
    DatasetId(String),
    TaxonId(String),
    AcceptedNameUsageId(String),
    ParentNameUsageId(String),
    ParentTaxon(String),

    ScientificName(String),
    Authorship(String),
    CanonicalName(String),
    AcceptedNameUsage(String),
    ParentNameUsage(String),

    TaxonomicRank(TaxonomicRank),
    TaxonomicStatus(TaxonomicStatus),
    NomenclaturalCode(String),
    NomenclaturalStatus(String),

    NamePublishedIn(String),
    NamePublishedInYear(String),
    NamePublishedInUrl(String),

    Citation(String),
    References(String),
    LastUpdated(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::taxa_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaxonOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: TaxonAtom,
}

#[derive(Queryable, Selectable, Debug, Deserialize, Clone)]
#[diesel(table_name = schema::taxa_logs)]
pub struct TaxonOperationWithDataset {
    #[diesel(embed)]
    pub operation: TaxonOperation,
    #[diesel(embed)]
    pub dataset_version: DatasetVersion,
    #[diesel(embed)]
    pub dataset: Dataset,
}

impl LogOperation<TaxonAtom> for TaxonOperationWithDataset {
    fn id(&self) -> &BigDecimal {
        self.operation.id()
    }

    fn entity_id(&self) -> &String {
        &self.operation.entity_id()
    }

    fn action(&self) -> &Action {
        self.operation.action()
    }

    fn atom(&self) -> &TaxonAtom {
        self.operation.atom()
    }
}

impl LogOperationDataset for TaxonOperationWithDataset {
    fn dataset_version(&self) -> &DatasetVersion {
        &self.dataset_version
    }

    fn dataset(&self) -> &Dataset {
        &self.dataset
    }
}


#[derive(Atom, Debug, Default, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum TaxonomicActAtom {
    #[default]
    Empty,
    EntityId(String),
    DatasetId(String),
    Publication(String),
    PublicationDate(String),
    Taxon(String),
    AcceptedTaxon(String),
    SourceUrl(String),
    CreatedAt(DateTime<Utc>),
    UpdatedAt(DateTime<Utc>),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::taxonomic_act_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaxonomicActOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: TaxonomicActAtom,
}


#[derive(Queryable, Selectable, Debug, Deserialize, Clone)]
#[diesel(table_name = schema::taxonomic_act_logs)]
pub struct TaxonomicActOperationWithDataset {
    #[diesel(embed)]
    pub operation: TaxonomicActOperation,
    #[diesel(embed)]
    pub dataset_version: DatasetVersion,
    #[diesel(embed)]
    pub dataset: Dataset,
}

impl LogOperation<TaxonomicActAtom> for TaxonomicActOperationWithDataset {
    fn id(&self) -> &BigDecimal {
        self.operation.id()
    }

    fn entity_id(&self) -> &String {
        &self.operation.entity_id()
    }

    fn action(&self) -> &Action {
        self.operation.action()
    }

    fn atom(&self) -> &TaxonomicActAtom {
        self.operation.atom()
    }
}


#[derive(Debug)]
pub enum NomenclaturalActTypeError {
    InvalidNomenclaturalActType(String),
}

impl TryFrom<&str> for NomenclaturalActType {
    type Error = NomenclaturalActTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use NomenclaturalActType::*;

        match value {
            "sp. nov." => Ok(SpeciesNova),
            "spec. nov." => Ok(SpeciesNova),
            "new species" => Ok(SpeciesNova),
            "comb. nov." => Ok(CombinatioNova),
            "stat. rev." => Ok(RevivedStatus),
            "gen. et sp. nov." => Ok(GenusSpeciesNova),
            "subsp. nov." => Ok(SubspeciesNova),
            val => Err(NomenclaturalActTypeError::InvalidNomenclaturalActType(val.to_string())),
        }
    }
}

#[derive(Atom, Debug, Default, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum NomenclaturalActAtom {
    #[default]
    Empty,
    EntityId(String),
    Publication(String),
    PublicationDate(String),
    ActedOn(String),
    Act(NomenclaturalActType),
    SourceUrl(String),

    ScientificName(String),
    Authorship(String),
    CanonicalName(String),
    AuthorityName(String),
    AuthorityYear(String),
    BasionymAuthorityName(String),
    BasionymAuthorityYear(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::nomenclatural_act_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NomenclaturalActOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: NomenclaturalActAtom,
}


#[derive(Atom, Debug, Clone, Default, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum TaxonDistributionAtom {
    #[default]
    Empty,

    /// Used to link the organism to a name
    ScientificName(String),

    EstablishmentMeans(String),
    DegreeOfEstablishment(String),
    Pathway(String),
}


#[derive(Atom, Debug, Clone, Default, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum SequenceAtom {
    #[default]
    Empty,
    EntityId(String),
    SequenceId(String),
    DnaExtractId(String),

    EventDate(String),
    EventTime(String),
    SequencedBy(String),
    MaterialSampleId(String),
    Concentration(String),
    AmpliconSize(i64),
    EstimatedSize(String),
    BaitSetName(String),
    BaitSetReference(String),
    TargetGene(String),
    DnaSequence(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::sequence_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SequenceOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: SequenceAtom,
}


#[derive(Atom, Debug, Clone, Default, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum PublicationAtom {
    #[default]
    Empty,
    EntityId(String),
    Title(String),
    Authors(Vec<String>),
    PublishedYear(i32),
    PublishedDate(DateTime<Utc>),
    Language(String),
    Publisher(String),
    Doi(String),
    SourceUrl(String),
    Type(PublicationType),
    Citation(String),
    RecordCreatedAt(DateTime<Utc>),
    RecordUpdatedAt(DateTime<Utc>),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::publication_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PublicationOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: PublicationAtom,
}


#[derive(Atom, Debug, Default, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum SpecimenAtom {
    #[default]
    Empty,

    /// The globally unique specimen id.
    SpecimenId(String),

    /// Used to link the collection to a name.
    ScientificName(String),

    /// When the specimen registration happened. Strictly YYYY-MM-DD
    EventDate(String),
    /// What time the specimen registration happened. Strictly HH:MM:SS
    EventTime(String),

    /// The name of the institution that owns the specimen.
    InstitutionName(String),
    /// The short code of the institution.
    InstitutionCode(String),
    /// The ID for the specific specimen. Typically the voucher registration number.
    CollectionRepositoryId(String),
    /// The code for the specific collection repository in the institution.
    CollectionRepositoryCode(String),

    TypeStatus(String),

    Preparation(String),

    OtherCatalogNumbers(String),

    Disposition(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::specimen_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SpecimenOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: SpecimenAtom,
}
