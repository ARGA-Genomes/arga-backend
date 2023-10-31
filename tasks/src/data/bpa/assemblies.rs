use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,
    bpa_library_id: Option<String>,
    library_id: Option<String>,
    assembly_date: Option<String>,
    assembly_method: Option<String>,
    assembly_method_version: Option<String>,
    assembly_method_version_or_date: Option<String>,
    assembly_name: Option<String>,
    genome_size: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssemblyEvent {
    id: String,
    sequence_id: String,
    event_date: Option<String>,
    assembly_type: Option<String>,
    version_status: Option<String>,
    name: Option<String>,
    genome_size: Option<String>,
}

impl AssemblyEvent {
    fn has_data(&self) -> bool {
        self.event_date.is_some() ||
            self.assembly_type.is_some() ||
            self.version_status.is_some() ||
            self.name.is_some() ||
            self.genome_size.is_some()
    }
}

pub fn normalise(path: &PathBuf) -> Result<(), Error> {
    let mut reader = csv::Reader::from_path(&path)?;
    let mut writer = csv::Writer::from_path("assemblies.csv")?;

    for row in reader.deserialize() {
        let record: Record = row?;

        let sequence_id = record
            .bpa_library_id
            .or(record.library_id.clone())
            .unwrap_or(record.id.clone());

        let version_status = record.assembly_method_version.or(record.assembly_method_version_or_date);

        let event = AssemblyEvent {
            id: record.id,
            sequence_id,
            event_date: record.assembly_date,
            assembly_type: record.assembly_method,
            version_status,
            name: record.assembly_name,
            genome_size: record.genome_size,
        };

        if event.has_data() {
            writer.serialize(event)?;
        }
    }

    Ok(())
}
