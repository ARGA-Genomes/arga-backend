use arga_core::schema::{names, taxa, taxon_names};
use diesel::prelude::*;


#[diesel::dsl::auto_type]
pub fn with_names() -> _ {
    names::table.on(names::id.eq(taxon_names::name_id))
}

#[diesel::dsl::auto_type]
pub fn with_taxon_names() -> _ {
    taxon_names::table.on(taxon_names::taxon_id.eq(taxa::id))
}

#[diesel::dsl::auto_type]
pub fn taxa_with_names() -> _ {
    taxa::table.inner_join(with_taxon_names()).inner_join(with_names())
}
