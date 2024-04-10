use std::collections::HashMap;

use crate::models::{Action, LogOperation};

// a map that uses the last-write-wins policy for each entry
#[derive(Debug)]
pub struct Map<T> {
    pub entity_id: String,
    pub atoms: HashMap<String, T>,
}

impl<T> Map<T>
where
    T: ToString + Clone + PartialEq + std::fmt::Debug,
{
    pub fn new(entity_id: String) -> Self {
        Self {
            entity_id,
            atoms: HashMap::new(),
        }
    }

    fn update(&mut self, atom: T) {
        let key = atom.to_string();
        self.atoms
            .entry(key.clone())
            .and_modify(|val| {
                if *val != atom {
                    *val = atom.clone();
                }
            })
            .or_insert(atom);
    }

    pub fn reduce<Op: LogOperation<T> + Clone>(&mut self, operations: &Vec<Op>) -> Vec<Op> {
        let mut inserts: HashMap<String, Op> = HashMap::new();
        let mut reduced: Vec<Op> = Vec::new();

        // this doesn't use versioned values and thus requires the operations
        // to be in causal order. that is to say that it should be ordered by
        // the id which in turn should be some representation of causality like
        // a timestamp
        for op in operations {
            match op.action() {
                Action::Create => self.update(op.atom().clone()),
                Action::Update => self.update(op.atom().clone()),
            }
        }

        // filter out operations that don't change the atom
        for op in operations {
            let key = op.atom().to_string();

            match inserts.get(&key) {
                Some(inserted_op) => {
                    if op.atom() != inserted_op.atom() {
                        reduced.push(op.clone());
                    }
                }
                None => {
                    reduced.push(op.clone());
                    inserts.insert(key, op.clone());
                }
            }
        }

        reduced
    }
}
