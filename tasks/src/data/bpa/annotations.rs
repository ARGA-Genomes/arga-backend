use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,
    genome_coverage: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnotationEvent {
    record_id: String,
    coverage: Option<String>,
}

pub fn normalise(path: &PathBuf) -> Result<(), Error> {
    let mut reader = csv::Reader::from_path(&path)?;
    let mut writer = csv::Writer::from_path("annotations.csv")?;

    for row in reader.deserialize() {
        let record: Record = row?;

        let event = AnnotationEvent {
            record_id: record.id,
            coverage: record.genome_coverage,
        };

        writer.serialize(event)?;
    }

    Ok(())
}
