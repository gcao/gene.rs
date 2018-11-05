extern crate gene;

use gene::parser::Parser;
use gene::types::Value;

#[test]
fn test_read_empty_input() {
    assert_eq!(Parser::new("").read(), Some(Ok(Value::Null)));
}

#[test]
fn test_read_number() {
    assert_eq!(Parser::new("1").read(), Some(Ok(Value::Integer(1))));
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
    assert_eq!(Parser::new("a").read(), Some(Ok(Value::Symbol("a".into()))));
}

#[test]
fn test_read_array() {
    assert_eq!(Parser::new("[]").read(), Some(Ok(Value::Array(vec![]))));
}
