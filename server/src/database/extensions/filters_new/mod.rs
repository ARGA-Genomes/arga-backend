pub mod name_attributes;
pub mod specimens;
pub mod stats;
pub mod taxa;


pub enum SortOrder {
    Ascending,
    Descending,
}

pub struct Sort<T> {
    pub sortable: T,
    pub order: SortOrder,
}
