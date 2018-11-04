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
    assert_eq!(Parser::new("ab cd").read_word(), Some(Ok("ab"))));

}