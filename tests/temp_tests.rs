// This file is used for debugging specific test
// It is referenced in .vscode/tasks.json like
// cargo test --no-run --message-format=json temp_tests

extern crate gene;

use gene::parser::Parser;
use gene::types::Value;

#[test]
fn test_read_array() {
    assert_eq!(Parser::new("[]").read(), Some(Ok(Value::Array(vec![]))));
}
