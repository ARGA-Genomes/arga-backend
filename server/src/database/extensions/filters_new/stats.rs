//! Filters for the taxa_tree_stats view
//!
//! Provides convenience functions that can be used in a diesel filter.
//! These filters are designed to enable a more generic approach to querying
//! the taxa_tree_stats view so that results can be arbitrarily based on
//! user input rather than pre-defined filtered endpoints.
//!
//! As a result all filters in here should maximise indicies and ensure that
//! sequence scans or n+1 queries are minimised.

use arga_core::schema::{name_attributes, taxa, taxon_names};
use arga_core::schema_gnl::taxa_tree_stats;
use diesel::prelude::*;
use uuid::Uuid;

use super::name_attributes::{has_attribute, Attribute, AttributeValueExpression};


// An alias of the `taxa` table.
// Useful when you need to join multiple columns to the taxa table. This is
// specifically used for taxa_tree_stats to refer to the `taxon_id` column.
diesel::alias!(taxa as root_node_taxa: RootNodeTaxa);


/// Include taxa data for the root node. (taxa_tree_stats.taxon_id)
#[diesel::dsl::auto_type]
pub fn with_root_node_taxa() -> _ {
    taxa::table.on(taxa::id.eq(taxa_tree_stats::taxon_id))
}

/// The same as `with_root_node_taxa` except it uses the aliased taxa table
/// `root_node_taxa` so that the descendant nodes can be joined on the taxa table itself.
#[diesel::dsl::auto_type]
pub fn with_aliased_root_node_taxa() -> _ {
    let root_node_id = root_node_taxa.field(taxa::id);
    root_node_taxa.on(root_node_id.eq(taxa_tree_stats::id))
}

/// Include taxa data for the descendant nodes. (taxa_tree_stats.id)
#[diesel::dsl::auto_type]
pub fn with_path_node_taxa() -> _ {
    taxa::table.on(taxa::id.eq(taxa_tree_stats::id))
}

/// The taxa_tree_stats view with accompanying taxa.
///
/// Joins the root taxon and the descendent taxa with the taxa table. The root taxon
/// is joined with the aliased table `root_node_taxa` so make sure to use that if you
/// want to get data related to the root node.
#[diesel::dsl::auto_type]
pub fn with_taxa() -> _ {
    taxa_tree_stats::table
        .inner_join(with_aliased_root_node_taxa())
        .inner_join(with_path_node_taxa())
}


/// The taxa_tree_stats view with accompanying name attributes.
///
/// Joins the root taxon to the name_attributes table via a through table. Since the root
/// taxon is usually the 'targeted' tree node we join it to the taxon_names through table
/// to get all attributes linked to a name used by the taxon.
/// This will reduce the result set largely to the species rank or below but it's important
/// to remember that an attribute can be assigned to any name, including higher taxonomy ones.
#[diesel::dsl::auto_type]
pub fn with_name_attributes() -> _ {
    // skip the taxa table and go straight to the taxon_names join table
    // since we actually just want names linked to a taxon since the attributes
    // themselves are on names.
    taxa_tree_stats::table
        .inner_join(taxon_names::table.on(taxon_names::taxon_id.eq(taxa_tree_stats::taxon_id)))
        .inner_join(name_attributes::table.on(name_attributes::name_id.eq(taxon_names::name_id)))
}


/// Filters taxa_tree_stats to taxa that are attributed to the specified dataset.
///
/// This will join the taxa_tree_stats table to the taxa table via the `taxon_id` field.
/// Because taxa trees are isolated to a single dataset we can guarantee that any taxa below
/// a taxon node is attributed to the same dataset. And because taxon_id is effectively the
/// 'root' node when you want to also retrieve descendants of the node we join on that field
/// to make things a bit simpler.
#[diesel::dsl::auto_type]
pub fn taxa_exist_in_dataset(dataset_id: Uuid) -> _ {
    let root_node_dataset_id = root_node_taxa.field(taxa::dataset_id);
    with_taxa().filter(root_node_dataset_id.eq(dataset_id))
}


#[diesel::dsl::auto_type]
pub fn taxa_has_attribute(attribute: Attribute) -> _ {
    let attr_filter: AttributeValueExpression = has_attribute(attribute);

    name_attributes::table.filter(attr_filter)

    // taxa_tree_stats::table
    //     .inner_join(taxon_names::table.on(taxon_names::taxon_id.eq(taxa_tree_stats::taxon_id)))
    //     .inner_join(attrs)
    // let attr_filter: AttributeValueExpression = has_attribute(attribute);
    // with_name_attributes().filter(attr_filter)
}
