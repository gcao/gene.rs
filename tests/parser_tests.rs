extern crate gene;

use ordered_float::OrderedFloat;
use std::collections::{BTreeMap};

use gene::parser::Parser;
use gene::types::Value;

#[test]
fn test_read_empty_input() {
    assert_eq!(Parser::new("").read(), Some(Ok(Value::Null)));
}

#[test]
fn test_read_number() {
    assert_eq!(Parser::new("1").read(), Some(Ok(Value::Integer(1))));
    assert_eq!(Parser::new("+1").read(), Some(Ok(Value::Integer(1))));
    assert_eq!(Parser::new("-1").read(), Some(Ok(Value::Integer(-1))));

    assert_eq!(Parser::new("1.1").read(), Some(Ok(Value::Float(OrderedFloat(1.1)))));
    assert_eq!(Parser::new("-1.1").read(), Some(Ok(Value::Float(OrderedFloat(-1.1)))));
}

#[test]
fn test_read_word() {
    assert_eq!(Parser::new("ab").read_word(), Some(Ok("ab".into())));
    assert_eq!(Parser::new("ab cd").read_word(), Some(Ok("ab".into())));
    assert_eq!(Parser::new("ab,cd").read_word(), Some(Ok("ab".into())));
}

#[test]
fn test_read_keywords() {
    assert_eq!(Parser::new("true").read(), Some(Ok(Value::Boolean(true))));
    assert_eq!(Parser::new(" true ").read(), Some(Ok(Value::Boolean(true))));
    assert_eq!(Parser::new("false").read(), Some(Ok(Value::Boolean(false))));
    assert_eq!(Parser::new("null").read(), Some(Ok(Value::Null)));
}

#[test]
fn test_read_string() {
    assert_eq!(Parser::new("\"ab\"").read(), Some(Ok(Value::String("ab".into()))));
}

#[test]
fn test_read_symbols() {
    assert_eq!(Parser::new("ab").read(), Some(Ok(Value::Symbol("ab".into()))));
}

#[test]
fn test_read_array() {
    assert_eq!(Parser::new("[]").read(), Some(Ok(Value::Array(vec![]))));
    assert_eq!(
        Parser::new("[1]").read(),
        Some(Ok(Value::Array(vec![Value::Integer(1)])))
    );
    assert_eq!(
        Parser::new("[1 2]").read(),
        Some(Ok(Value::Array(vec![Value::Integer(1), Value::Integer(2)])))
    );
}

#[test]
fn test_read_map() {
    assert_eq!(Parser::new("{}").read(), Some(Ok(Value::Map(BTreeMap::new()))));
    {
        let mut map = BTreeMap::new();
        map.insert("key".into(), Value::Integer(123));
        assert_eq!(
            Parser::new("{^key 123}").read(),
            Some(Ok(Value::Map(map)))
        );
    }
    {
        let mut map = BTreeMap::new();
        let arr = Value::Array(vec![Value::Integer(123)]);
        map.insert("key".into(), arr);
        assert_eq!(
            Parser::new("{^key [123]}").read(),
            Some(Ok(Value::Map(map)))
        );
    }
}
