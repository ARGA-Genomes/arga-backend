use async_graphql::{InputObject, OneofObject};
use bigdecimal::FromPrimitive;

use crate::database::extensions::filters_new::name_attributes::AttributeValue;


#[derive(Debug, Clone, InputObject)]
pub struct NameAttributeInput {
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(OneofObject)]
pub enum NameAttributeFilter {
    Attribute(NameAttributeInput),
    AttributeName(String),
}


pub enum JsonAttributeValueError {
    CannotBeNull,
    InvalidNumber(String),
    InvalidFloat(String),
    ArrayUnsupported,
    ObjectUnsupported,
}


impl TryFrom<serde_json::Value> for AttributeValue {
    type Error = JsonAttributeValueError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        match value {
            serde_json::Value::Null => Err(Self::Error::CannotBeNull),
            serde_json::Value::Bool(value) => Ok(AttributeValue::Boolean(value)),
            serde_json::Value::Number(number) => {
                // numbers in json don't map neatly into lower level types
                // for numbers. so we see if it can be converted into an integer and
                // failing that try converting it into a float.
                //
                // this isn't ideal since we are using floats to map into a decimal
                // value but it'll do for now until we have a more precise number type
                // for the api
                if number.is_i64() {
                    let value = number.as_i64();
                    match value {
                        Some(value) => Ok(AttributeValue::Integer(value)),
                        None => Err(Self::Error::InvalidNumber(number.to_string())),
                    }
                }
                else if number.is_f64() {
                    let value = number.as_f64();
                    match value {
                        Some(value) => match bigdecimal::BigDecimal::from_f64(value) {
                            Some(decimal) => Ok(AttributeValue::Decimal(decimal)),
                            None => Err(Self::Error::InvalidFloat(value.to_string())),
                        },
                        None => Err(Self::Error::InvalidFloat(number.to_string())),
                    }
                }
                else {
                    Err(Self::Error::InvalidNumber(number.to_string()))
                }
            }
            serde_json::Value::String(value) => Ok(AttributeValue::String(value)),
            serde_json::Value::Array(_values) => Err(Self::Error::ArrayUnsupported),
            serde_json::Value::Object(_map) => Err(Self::Error::ObjectUnsupported),
        }
    }
}
