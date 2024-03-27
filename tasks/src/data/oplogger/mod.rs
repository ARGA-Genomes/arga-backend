pub mod hlc;
pub mod nomenclatural_acts;

use arga_core::models::{Action, Atom, Operation};
use chrono::Utc;
use std::{collections::HashMap, path::PathBuf};

use self::hlc::HybridTimestamp;

#[derive(Debug)]
pub struct Version(HybridTimestamp);

impl Version {
    /// Get the next frame version.
    ///
    /// This will generate a new hybrid logical clock and if it is greater
    /// than the current version it will return it. However, if the current
    /// clock is ahead we keep incrementing it.
    pub fn next(&self) -> Version {
        let ts: HybridTimestamp = Utc::now().into();
        if ts > self.0 {
            Version(ts)
        } else {
            Version(self.0.inc())
        }
    }
}

#[derive(Debug)]
pub struct ObjectFrame {
    pub previous: Version,
    pub object_id: String,
    pub operations: Vec<Operation>,
}

impl ObjectFrame {
    pub fn new(version: Version, object_id: String) -> ObjectFrame {
        let next = version.next();

        let creation = Operation {
            operation_id: next.0.as_u64().into(),
            object_id: object_id.clone(),
            reference_id: version.0.as_u64().into(),
            action: Action::Create,
            atom: Atom::Empty,
        };

        ObjectFrame {
            previous: next,
            object_id,
            operations: vec![creation],
        }
    }

    pub fn update(&mut self, atom: Atom) {
        let last_op = self.operations.last().unwrap();
        let next = self.previous.0.inc();

        let op = Operation {
            operation_id: next.as_u64().into(),
            object_id: last_op.object_id.clone(),
            reference_id: self.previous.0.as_u64().into(),
            action: Action::Update,
            atom,
        };

        self.operations.push(op);
        self.previous.0 = next;
    }
}

// a map that uses the last-write-wins policy for each entry
#[derive(Debug)]
pub struct LWWMap {
    pub entity_id: String,
    pub map: HashMap<String, Atom>,
}

impl LWWMap {
    pub fn new(entity_id: String) -> Self {
        Self {
            entity_id,
            map: HashMap::new(),
        }
    }

    fn update(&mut self, atom: Atom) {
        self.map.insert(atom.to_string(), atom);
    }

    pub fn reduce(&mut self, operations: &Vec<Operation>) {
        // this doesn't use versioned values an thus requires the operations
        // to be in causal order. that is to say that it should be ordered by
        // the id which in turn should be some representation of causality like
        // a timestamp
        for op in operations {
            match op.action {
                Action::Create => {}
                Action::Update => self.update(op.atom.clone()),
            }
        }
    }
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Process and import a csv as operation logs
    #[command(subcommand)]
    Import(ImportCommand),

    /// Reduce operation logs into entity objects
    #[command(subcommand)]
    Reduce(ReduceCommand),
}

#[derive(clap::Subcommand)]
pub enum ImportCommand {
    /// Extract nomenclatural acts from a CSV dataset
    NomenclaturalActs { path: PathBuf },
}

#[derive(clap::Subcommand)]
pub enum ReduceCommand {
    NomenclaturalActs,
}

pub fn process_command(cmd: &Command) {
    match cmd {
        Command::Import(cmd) => match cmd {
            ImportCommand::NomenclaturalActs { path } => {
                nomenclatural_acts::process(path.clone()).unwrap()
            }
        },
        Command::Reduce(cmd) => match cmd {
            ReduceCommand::NomenclaturalActs => nomenclatural_acts::reduce().unwrap(),
        },
    }
}
