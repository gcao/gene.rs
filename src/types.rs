extern crate ordered_float;

use ordered_float::OrderedFloat;
use std::collections::{BTreeMap};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(OrderedFloat<f64>),
    String(String),
    Symbol(String),
    Array(Vec<Value>),
    Map(BTreeMap<String, Value>),
}

pub struct Pair {
    pub key: String,
    pub val: Value,
}

impl Pair {
    pub fn new(key: String, val: Value) ->Pair {
        return Pair {
            key: key,
            val: val,
        };
    }
}