// This file is used for debugging specific test
// It is referenced in .vscode/tasks.json like
// cargo test --no-run --message-format=json temp_tests

extern crate gene;
extern crate ordered_float;

use std::collections::BTreeMap;

use gene::parser::{Error, Parser};
use gene::Value;
use gene::types::Gene;

#[test]
fn test_this() {
    assert_eq!(Parser::new("ab").read_word(), Some(Ok("ab"))));
    assert_eq!(Parser::new("ab cd").read_word(), Some(Ok("ab"))));

    let mut parser = Parser::new(
        "{} {^a 1} {^a 1 ^b 2} {^^a} {^!a} {^^a ^b 1}",
    );

    assert_eq!(parser.read(), Some(Ok(Value::Map(BTreeMap::new()))));

    assert_eq!(
        parser.read(),
        Some(Ok(Value::Map({
            let mut map = BTreeMap::new();
            map.insert(Value::String("a".into()), Value::Integer(1));
            map
        })))
    );

    let mut parser = Parser::new("{\\foo true}");
    assert_eq!(
        parser.read(),
        Some(Err(Error {
            lo: 1,
            hi: 5,
            message: "invalid char literal `\\foo`".into()
        }))
    );

    let mut parser = Parser::new("{ { 1 2 3");
    assert_eq!(
        parser.read(),
        Some(Err(Error {
            lo: 2,
            hi: 9,
            message: "unclosed `{`".into()
        }))
    );

    let mut parser = Parser::new("{1 2 3}");
    assert_eq!(
        parser.read(),
        Some(Err(Error {
            lo: 0,
            hi: 7,
            message: "odd number of items in a Map".into()
        }))
    );

    let mut parser = Parser::new("{{1 2 3}}");
    assert_eq!(
        parser.read(),
        Some(Err(Error {
            lo: 1,
            hi: 8,
            message: "odd number of items in a Map".into()
        }))
    );
}