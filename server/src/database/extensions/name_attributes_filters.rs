use arga_core::models::AttributeValueType;
use arga_core::schema::name_attributes;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;

use super::filters::{AttributeValue, NameAttribute};

type BoxedExpression<'a> =
    Box<dyn BoxableExpression<name_attributes::table, Pg, SqlType = diesel::sql_types::Nullable<Bool>> + 'a>;

#[derive(Clone, Debug)]
pub enum NameAttributeFilter {
    NameAttribute(NameAttribute),
}

pub fn with_name_attribute(name_attribute: &NameAttributeFilter) -> BoxedExpression {
    match name_attribute {
        NameAttributeFilter::NameAttribute(attribute) => {
            // Build a BoxedExpression for each match arm
            let expr: Box<
                dyn BoxableExpression<name_attributes::table, Pg, SqlType = diesel::sql_types::Nullable<Bool>>,
            > = match attribute.value_type {
                AttributeValueType::Boolean => Box::new(name_attributes::value_bool.eq(match &attribute.value {
                    AttributeValue::Boolean(b) => *b,
                    _ => false,
                })),
                AttributeValueType::Integer => Box::new(name_attributes::value_int.eq(match &attribute.value {
                    AttributeValue::Integer(i) => *i,
                    _ => 0,
                })),
                AttributeValueType::Decimal => Box::new(name_attributes::value_decimal.eq(match &attribute.value {
                    AttributeValue::Decimal(d) => d.clone(),
                    _ => Default::default(),
                })),
                AttributeValueType::String => Box::new(name_attributes::value_str.eq(match &attribute.value {
                    AttributeValue::String(s) => s.clone(),
                    _ => "".to_string(),
                })),
                AttributeValueType::Timestamp => {
                    Box::new(name_attributes::value_timestamp.eq(match &attribute.value {
                        AttributeValue::Timestamp(ts) => ts.clone(),
                        _ => Default::default(),
                    }))
                }
            };

            // Then apply the final `.and(...)` clause once
            Box::new(expr.and(name_attributes::name.eq(attribute.name.clone())))
        }
    }
}
