extern crate ordered_float;

use std::clone::Clone;
use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

use ordered_float::OrderedFloat;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    Gene(Box<Gene>),
    Stream(Vec<Value>),
}

impl Clone for Value {
    fn clone(&self) -> Value {
        match &self {
            Value::Void => Value::Void,
            Value::Null => Value::Null,
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Integer(i) => Value::Integer(*i),
            Value::Float(f) => Value::Float(*f),
            Value::String(s) => Value::String(s.clone()),
            Value::Symbol(symbol) => Value::Symbol(symbol.clone()),
            Value::Array(a) => Value::Array(a.to_vec()),
            Value::Map(m) => {
                let mut new_map = BTreeMap::new();
                for (k, v) in m.iter() {
                    new_map.insert(k.to_string(), v.clone());
                }
                Value::Map(new_map)
            }
            Value::Gene(g) => Value::Gene(g.clone()),
            Value::Stream(v) => Value::Stream(v.to_vec()),
        }
    }
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
            Value::Symbol(v) => {
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
    pub kind: Value,
    pub props: BTreeMap<String, Rc<RefCell<Value>>>,
    pub data: Vec<Value>,
}

impl Gene {
    pub fn new(kind: Value) -> Self {
        Gene {
            kind: kind,
            props: BTreeMap::new(),
            data: Vec::new(),
        }
    }
}

impl fmt::Display for Gene {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("(")?;
        fmt.write_str(&self.kind.to_string())?;
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
        Pair { key, val }
    }
}
