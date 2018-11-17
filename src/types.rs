extern crate ordered_float;

use ordered_float::OrderedFloat;
use std::fmt;
use std::collections::{BTreeMap};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Void, // Same as undefined, different from null, can be represented as ()
    Null, // Default value for any type, equivalent to false, 0, "", [], {}, (null) etc
    Boolean(bool),
    Integer(i64),
    Float(OrderedFloat<f64>),
    String(String),
    Symbol(String),
    Array(Vec<Value>),
    Map(BTreeMap<String, Value>),
    Gene(Gene),
    Stream(Vec<Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Value::Void => {
                fmt.write_str("()")?;
            }
            Value::Null => {
                fmt.write_str("null")?;
            }
            Value::Boolean(true) => {
                fmt.write_str("true")?;
            }
            Value::Boolean(false) => {
                fmt.write_str("false")?;
            }
            Value::Integer(v) => {
                fmt.write_str(&v.to_string())?;
            }
            Value::String(v) => {
                fmt.write_str(&v)?;
            }
            Value::Gene(v) => {
                fmt.write_str(&v.to_string())?;
            }
            _ => {
                fmt.write_str("(fmt to be implemented)")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gene {
    pub _type: Box<Value>,
    pub props: BTreeMap<String, Box<Value>>,
    pub data: Vec<Box<Value>>,
}

impl Gene {
    pub fn new(_type: Value) -> Gene {
        return Gene {
            _type: Box::new(_type),
            props: BTreeMap::new(),
            data: vec![],
        }
    }
}

impl fmt::Display for Gene {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("(")?;
        fmt.write_str(&self._type.to_string())?;
        fmt.write_str(" ...)")?;
        Ok(())
    }
}

pub struct Pair {
    pub key: String,
    pub val: Value,
}

impl Pair {
    pub fn new(key: String, val: Value) -> Self {
        return Pair {
            key: key,
            val: val,
        };
    }
}