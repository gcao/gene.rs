extern crate ordered_float;

use ordered_float::OrderedFloat;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Todo,
    Null,
    Boolean(bool),
    Integer(i64),
    Float(OrderedFloat<f64>),
    String(String),
    Symbol(String),
    Array(Vec<Value>),
}
