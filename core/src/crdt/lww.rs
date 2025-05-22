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
    T: ToString + Clone + PartialEq,
{
    pub fn new(entity_id: String) -> Self {
        Self {
            entity_id,
            atoms: HashMap::new(),
        }
    }

    // this doesn't use versioned values and thus requires the operations
    // to be in causal order. that is to say that it should be ordered by
    // the id which in turn should be some representation of causality like
    // a timestamp.
    // because reduction could be called *a lot* of times on big databases we
    // deal take and return references with the same lifetime to avoid cloning
    pub fn reduce<'a, Op: LogOperation<T> + Clone>(&mut self, operations: &'a Vec<Op>) -> Vec<&'a Op> {
        let mut inserts: HashMap<String, &Op> = HashMap::new();

        // filter out operations that don't change the atom
        for op in operations {
            // the atom descriminant
            let key = op.atom().to_string();

            // find out if an operation has previously set this field. if so then we only
            // want to include ones that change the _last_ operation affecting that field.
            // in this way we can make sure that values that change back and forth will still
            // be included in the log.
            inserts
                .entry(key)
                .and_modify(|old_op| {
                    // atoms must implement PartialEq so what is considered
                    // a change depends on the trait implementation. for the most part
                    // this will be a variant so the value will also be tested for equality
                    if old_op.atom() != op.atom() {
                        *old_op = op;
                    }
                })
                .or_insert(op);
        }

        inserts.into_values().collect()
    }
}
