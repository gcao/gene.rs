extern crate ordered_float;

use ordered_float::OrderedFloat;
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gene {
    pub Type: Box<Value>,
    pub props: BTreeMap<String, Box<Value>>,
    pub data: Vec<Box<Value>>,
}

impl Gene {
    pub fn new(Type: Value) -> Gene {
        return Gene {
            Type: Box::new(Type),
            props: BTreeMap::new(),
            data: vec![],
        }
    }
}

pub struct Pair {
    pub key: String,
    pub val: Value,
}

impl Pair {
    pub fn new(key: String, val: Value) -> Pair {
        return Pair {
            key: key,
            val: val,
        };
    }
}