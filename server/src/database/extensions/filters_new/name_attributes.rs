use arga_core::schema::name_attributes;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable};


/// A boxable expression for the Entity-Attribute-Value table.
///
/// An EAV table allows us to associate arbitrary values to a name by having a column
/// holding the type and the value stored in a typed column, such as `value_int` or `value_str`.
/// This makes it tricky to create a query from one variant so we define a boxed type for
/// query helpers that can match on the variant and return an appropriate where condition while
/// still remaining type safe.
pub type AttributeValueExpression = Box<dyn BoxableExpression<name_attributes::table, Pg, SqlType = Nullable<Bool>>>;


pub enum Attribute {
    String(String),
    Boolean(bool),
    Integer(i64),
    Decimal(bigdecimal::BigDecimal),
    Timestamp(chrono::NaiveDateTime),
}


pub fn has_attribute(attribute: Attribute) -> AttributeValueExpression {
    match attribute {
        Attribute::String(value) => Box::new(name_attributes::value_str.eq(value)),
        Attribute::Boolean(value) => Box::new(name_attributes::value_bool.eq(value)),
        Attribute::Integer(value) => Box::new(name_attributes::value_int.eq(value)),
        Attribute::Decimal(value) => Box::new(name_attributes::value_decimal.eq(value)),
        Attribute::Timestamp(value) => Box::new(name_attributes::value_timestamp.eq(value)),
    }
}
