pub mod hlc;
pub mod nomenclatural_acts;

use std::path::PathBuf;
use arga_core::models::{Operation, Action, Atom};
use chrono::Utc;

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
            atom: None,
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
            atom: Some(serde_json::to_value(atom).unwrap()),
        };

        self.operations.push(op);
        self.previous.0 = next;
    }
}


#[derive(clap::Subcommand)]
pub enum Command {
    /// Extract nomenclatural acts from a CSV dataset
    NomenclaturalActs { path: PathBuf },
}

pub fn process_command(cmd: &Command) {
    match cmd {
        Command::NomenclaturalActs { path } => nomenclatural_acts::process(path.clone()).unwrap(),
    }
}
