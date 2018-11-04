// This file is used for debugging specific test
// It is referenced in .vscode/tasks.json like
// cargo test --no-run --message-format=json temp_tests

extern crate gene;

use gene::parser::Parser;

#[test]
fn test_read_word() {
    assert_eq!(Parser::new("ab").read_word(), Some(Ok("ab".into())));
}
