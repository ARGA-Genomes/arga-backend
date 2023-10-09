use async_graphql::Enum;
use serde::{Serialize, Deserialize};

use crate::database::extensions::whole_genome_filters;


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "whole_genome_filters::AssemblyLevel")]
pub enum AssemblyLevel {
    CompleteGenome,
    Chromosome,
    Scaffold,
    Contig,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "whole_genome_filters::GenomeRepresentation")]
pub enum GenomeRepresentation {
    Complete,
    Full,
    Partial,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "whole_genome_filters::ReleaseType")]
pub enum ReleaseType {
    Major,
    Minor,
    Patch,
}
