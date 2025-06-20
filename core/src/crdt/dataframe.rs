use bigdecimal::BigDecimal;
use uuid::Uuid;

use super::Version;
use crate::models::logs::Action;


#[derive(Debug, Clone)]
pub struct DataFrameOperation<Atom> {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: Atom,
}

#[derive(Debug, Clone)]
pub struct DataFrame<Atom> {
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub operations: Vec<DataFrameOperation<Atom>>,

    current: Version,
    next: Version,
}

impl<Atom: Default> DataFrame<Atom> {
    pub fn create(entity_id: String, dataset_version_id: uuid::Uuid, last_version: Version) -> DataFrame<Atom> {
        let next = last_version.next();
        let current = last_version;

        // let creation_op = DataFrameOperation {
        //     operation_id: next.into(),
        //     parent_id: current.into(),
        //     dataset_version_id,
        //     entity_id: entity_id.clone(),
        //     action: Action::Create,
        //     atom: Atom::default(),
        // };

        DataFrame {
            entity_id,
            dataset_version_id,
            operations: vec![],
            next: last_version.next(),
            current: last_version,
        }
    }

    pub fn last_version(&self) -> Version {
        self.current
    }

    pub fn push_operation(&mut self, operation: DataFrameOperation<Atom>) -> Version {
        self.operations.push(operation);

        // increment the version for the next op if there is one. we could do
        // a next() instead but frames are transactional so having them incrementing
        // logically is more consistent
        self.current = self.next;
        self.next = self.next.inc();
        self.next
    }

    pub fn push(&mut self, atom: Atom) {
        let operation_id: BigDecimal = self.next.into();
        let parent_id = self
            .operations
            .last()
            .map(|op| op.operation_id.clone())
            .unwrap_or(operation_id.clone());

        let operation = DataFrameOperation {
            operation_id,
            parent_id,
            dataset_version_id: self.dataset_version_id,
            entity_id: self.entity_id.clone(),
            action: Action::Update,
            atom,
        };

        self.push_operation(operation);
    }

    pub fn collect<T>(self) -> Vec<T>
    where
        T: From<DataFrameOperation<Atom>>,
    {
        self.operations.into_iter().map(|op| op.into()).collect()
    }
}


impl<Atom: std::fmt::Debug> std::fmt::Display for DataFrame<Atom> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for op in self.operations.iter() {
            f.write_fmt(format_args!(
                "{} {} {} {} {:?}\n",
                op.operation_id, op.parent_id, op.entity_id, op.action, op.atom
            ))?;
        }
        Ok(())
    }
}
