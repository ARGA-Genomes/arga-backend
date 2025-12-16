use bigdecimal::BigDecimal;
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

use super::{Action, LogOperation};
use crate::crdt::DataFrameOperation;
use crate::models::{DatasetVersion, schema};


#[derive(Atom, Debug, Clone, Default, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum ProjectAtom {
    #[default]
    Empty,

    ProjectId(String),
    TargetSpecies(String),
    Title(String),
    Description(String),
    Initiative(String),
    InitiativeTheme(String),
    RegistrationDate(chrono::NaiveDate),

    DataContext(Vec<String>),
    DataTypes(Vec<String>),
    DataAssayTypes(Vec<String>),
    Partners(Vec<String>),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::project_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProjectOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: ProjectAtom,
}
