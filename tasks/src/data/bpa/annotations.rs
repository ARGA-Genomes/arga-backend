use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,
    bpa_library_id: Option<String>,
    library_id: Option<String>,
    genome_coverage: Option<String>,
    sequence_data_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnotationEvent {
    id: String,
    sequence_id: String,
    coverage: Option<String>,
    genome_representation: Option<String>,
    data_type: Option<String>,
}

impl AnnotationEvent {
    fn has_data(&self) -> bool {
        self.coverage.is_some() ||
            self.genome_representation.is_some() ||
            self.data_type.is_some()
    }
}

pub fn normalise(path: &PathBuf) -> Result<(), Error> {
    let mut reader = csv::Reader::from_path(&path)?;
    let mut writer = csv::Writer::from_path("annotations.csv")?;

    for row in reader.deserialize() {
        let record: Record = row?;

        let sequence_id = record
            .bpa_library_id
            .or(record.library_id.clone())
            .unwrap_or(record.id.clone());

        let (data_type, representation) = match record.sequence_data_type.as_ref().map(|s| s.as_str()) {
            Some("illumina-amplicons") => (Some("marker"), None),
            Some("illumina-shortread") => (Some("whole_genome"), Some("Partial")),
            Some("illumina-exoncapture") => (Some("whole_genome"), Some("Partial")),
            Some("metabolomics") => (Some("metabolome"), None),
            Some("proteomics") => (Some("proteome"), None),
            Some("illumina-transcriptomics") => (Some("transcriptome"), None),
            Some("pacbio-rsii") => (Some("whole_genome"), Some("Complete")),
            Some("pacbio-hifi") => (Some("whole_genome"), Some("Complete")),
            Some("Illumina-shortread") => (Some("whole_genome"), Some("Complete")),
            Some("image") => (Some("image"), None),
            Some("ont-promethion") => (Some("whole_genome"), Some("Complete")),
            Some("transcriptomics-analysed") => (Some("transcriptome"), None),
            Some("illumina-ddrad") => (Some("snps_ddrad"), None),
            Some("Illumina-transcriptomics") => (Some("transcriptome"), None),
            Some("metagenomics-analysed") => (Some("metagenome"), None),
            Some("illumina-hic") => (Some("whole_genome"), Some("Complete")),
            Some("pacbio-clr") => (None, None),
            Some("illumina-10x") => (Some("whole_genome"), Some("Complete")),
            Some("illumina_transcriptomics") => (Some("transcriptome"), None),
            Some("analysed-data") => (None, None),
            Some("illumina-dart") => (Some("snps_ddran"), None),
            Some("metabolomics-analysed") => (Some("metabolome"), None),
            Some("proteomics-analysed") => (Some("proteome"), None),
            Some("ont-minion") => (Some("whole_genome"), None),
            Some("genome-assembly") => (Some("whole_genome"), Some("Complete")),
            _ => (None, None),
        };

        let event = AnnotationEvent {
            id: record.id,
            sequence_id,
            coverage: record.genome_coverage,
            genome_representation: representation.map(|s| s.to_string()),
            data_type: data_type.map(|s| s.to_string()),
        };

        if event.has_data() {
            writer.serialize(event)?;
        }
    }

    Ok(())
}
