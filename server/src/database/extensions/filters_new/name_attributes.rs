use arga_core::schema::{name_attributes, taxon_names};
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


#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

impl Attribute {
    pub fn new(name: &str, value: AttributeValue) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Boolean(bool),
    Integer(i64),
    Decimal(bigdecimal::BigDecimal),
    Timestamp(chrono::NaiveDateTime),
}

impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<bool> for AttributeValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i64> for AttributeValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<bigdecimal::BigDecimal> for AttributeValue {
    fn from(value: bigdecimal::BigDecimal) -> Self {
        Self::Decimal(value)
    }
}

impl From<chrono::NaiveDateTime> for AttributeValue {
    fn from(value: chrono::NaiveDateTime) -> Self {
        Self::Timestamp(value)
    }
}


pub fn has_attribute_value(attribute_value: AttributeValue) -> AttributeValueExpression {
    match attribute_value {
        AttributeValue::String(value) => Box::new(name_attributes::value_str.eq(value)),
        AttributeValue::Boolean(value) => Box::new(name_attributes::value_bool.eq(value)),
        AttributeValue::Integer(value) => Box::new(name_attributes::value_int.eq(value)),
        AttributeValue::Decimal(value) => Box::new(name_attributes::value_decimal.eq(value)),
        AttributeValue::Timestamp(value) => Box::new(name_attributes::value_timestamp.eq(value)),
    }
}

#[diesel::dsl::auto_type]
pub fn has_attribute_name(attribute_name: String) -> _ {
    name_attributes::name.eq(attribute_name)
}

#[diesel::dsl::auto_type]
pub fn has_attribute<'a>(attribute: Attribute) -> _ {
    let query: diesel::dsl::IntoBoxed<'a, with_attribute_value, Pg> =
        with_attribute_value(attribute.value).into_boxed();

    let name: has_attribute_name = has_attribute_name(attribute.name);
    query.filter(name)
}


/// Join taxon_names to name_attributes
#[diesel::dsl::auto_type]
pub fn with_taxon_names() -> _ {
    taxon_names::table.on(taxon_names::name_id.eq(name_attributes::name_id))
}


#[diesel::dsl::auto_type]
pub fn with_attribute_value(value: AttributeValue) -> _ {
    let attr_filter: AttributeValueExpression = has_attribute_value(value);
    name_attributes::table.filter(attr_filter)
}

#[diesel::dsl::auto_type]
pub fn with_taxa_attribute<'a>(attribute: Attribute) -> _ {
    // auto_type struggles to find the right type for our boxed subquery so
    // we explicitly type it here. we need to box the query because the has_attribute_value
    // is a boxable expression for the name_attributes table alone, not for name_attributes
    // joined with the taxon_names table.
    let query: diesel::dsl::IntoBoxed<'a, with_attribute_value, Pg> = with_attribute_value(attribute.value)
        .filter(name_attributes::name.eq(attribute.name))
        .into_boxed();

    query.inner_join(with_taxon_names())
}
