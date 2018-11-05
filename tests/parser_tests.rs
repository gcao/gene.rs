extern crate gene;

use gene::parser::Parser;
use gene::types::Value;

#[test]
fn test_read_word() {
    assert_eq!(Parser::new("ab").read_word(), Some(Ok("ab".into())));
    assert_eq!(Parser::new("ab cd").read_word(), Some(Ok("ab".into())));
    assert_eq!(Parser::new("ab,cd").read_word(), Some(Ok("ab".into())));
}

#[test]
fn test_read_keywords() {
    assert_eq!(Parser::new("true").read(), Some(Ok(Value::Boolean(true))));
    assert_eq!(Parser::new("true false").read(), Some(Ok(Value::Boolean(true))));
    assert_eq!(Parser::new("false").read(), Some(Ok(Value::Boolean(false))));
    assert_eq!(Parser::new("null").read(), Some(Ok(Value::Null)));
}
