// This file is used for debugging specific test
// It is referenced in .vscode/tasks.json like
// cargo test --no-run --message-format=json temp_tests

extern crate ordered_float;
extern crate gene;

use ordered_float::OrderedFloat;

use gene::parser::Parser;
use gene::types::Value;

#[test]
fn test_read_array() {
    // assert_eq!(Parser::new("[1]").read(),
    //   Some(Ok(Value::Array(vec![Value::Integer(1)])))
    // );
}
