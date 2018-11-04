extern crate gene;

use gene::parser::Parser;

#[test]
fn test_read_word() {
    assert_eq!(Parser::new("ab").read_word(), Some(Ok("ab".into())));
}