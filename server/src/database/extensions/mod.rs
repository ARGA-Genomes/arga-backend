pub mod filters_new;

pub mod classification_filters;
pub mod column_sum;
pub mod filters;
pub mod pagination;
pub mod species_filters;
pub mod taxa_filters;
pub mod whole_genome_filters;

pub use column_sum::sum_if;
use diesel::expression::functions::define_sql_function;
use diesel::sql_types::{Date, Double, Nullable, Text};
pub use pagination::{FilteredPage, Page, Paginate};

define_sql_function! {
    /// Returns a lowercase version of the text
    fn lower(text: Text) -> Text;
}

define_sql_function! {
    /// Returns a lowercase version of the text. Accepts null values
    #[sql_name = "lower"]
    fn lower_opt(text: Nullable<Text>) -> Nullable<Text>;
}

define_sql_function! {
    /// Returns an uppercase version of the text
    fn upper(text: Text) -> Text;
}

define_sql_function! {
    /// Returns an uppercase version of the text. Accepts null values
    #[sql_name = "upper"]
    fn upper_opt(text: Nullable<Text>) -> Nullable<Text>;
}

define_sql_function! {
    /// Extracts a component from a date
    fn date_part(field: Text, source: Date) -> Double;
}
