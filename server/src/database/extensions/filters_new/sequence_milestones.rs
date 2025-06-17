//! Filters for the sequence_milestones view
//!
//! Provides convenience functions that can be used in a diesel filter.
//! These filters are designed to enable a more generic approach to querying
//! the sequence_milestones view so that results can be arbitrarily based on
//! user input rather than pre-defined filtered endpoints.
//!
//! As a result all filters in here should maximise indicies and ensure that
//! sequence scans or n+1 queries are minimised.

use arga_core::schema::taxon_names;
use arga_core::schema_gnl::sequence_milestones;
use diesel::prelude::*;


#[diesel::dsl::auto_type]
pub fn with_taxon_names() -> _ {
    sequence_milestones::table.on(taxon_names::name_id.eq(sequence_milestones::name_id))
}

#[diesel::dsl::auto_type]
pub fn sequence_milestones_with_taxa() -> _ {

}
