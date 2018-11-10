extern crate gene;

use ordered_float::OrderedFloat;
use std::collections::{BTreeMap};

use gene::parser::Parser;
use gene::types::Value;
use gene::types::Gene;

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

// read_word() is not a public method, should not be tested directly
// If it has to be tested, parser.next() should be called first.
// #[test]
// fn test_read_word() {
//     assert_eq!(Parser::new("ab").read_word(), Some(Ok("ab".into())));
//     assert_eq!(Parser::new("ab cd").read_word(), Some(Ok("ab".into())));
//     assert_eq!(Parser::new("ab,cd").read_word(), Some(Ok("ab".into())));
//     assert_eq!(Parser::new("你好").read_word(), Some(Ok("你好".into())));
// }

#[test]
fn test_read_keywords() {
    assert_eq!(Parser::new("true").read(), Some(Ok(Value::Boolean(true))));
    assert_eq!(Parser::new(" true ").read(), Some(Ok(Value::Boolean(true))));
    assert_eq!(Parser::new("false").read(), Some(Ok(Value::Boolean(false))));
    assert_eq!(Parser::new("null").read(), Some(Ok(Value::Null)));
    assert_eq!(Parser::new("\\true").read(), Some(Ok(Value::Symbol("true".into()))));
}

#[test]
fn test_read_string() {
    assert_eq!(Parser::new("\"ab\"").read(), Some(Ok(Value::String("ab".into()))));
    assert_eq!(Parser::new("\"a\nb\"").read(), Some(Ok(Value::String("a\nb".into()))));
    assert_eq!(Parser::new("\"ab \\\"cd\\\"\"").read(), Some(Ok(Value::String("ab \"cd\"".into()))));
    assert_eq!(Parser::new("\"你好\"").read(), Some(Ok(Value::String("你好".into()))));
}

#[test]
fn test_skip_comment() {
    assert_eq!(Parser::new("#\nab").read(), Some(Ok(Value::Symbol("ab".into()))));
    assert_eq!(Parser::new("#!test\nab").read(), Some(Ok(Value::Symbol("ab".into()))));
}

#[test]
fn test_read_symbols() {
    assert_eq!(Parser::new("ab").read(), Some(Ok(Value::Symbol("ab".into()))));
    assert_eq!(Parser::new("你好").read(), Some(Ok(Value::Symbol("你好".into()))));
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
        map.insert("key".into(), Value::Boolean(true));
        assert_eq!(
            Parser::new("{^^key}").read(),
            Some(Ok(Value::Map(map)))
        );
    }
    {
        let mut map = BTreeMap::new();
        map.insert("key".into(), Value::Boolean(false));
        assert_eq!(
            Parser::new("{^!key}").read(),
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

#[test]
fn test_read_gene() {
    assert_eq!(Parser::new("()").read(), Some(Ok(Value::Gene(Gene::new(Value::Null)))));
    {
        let result = Gene::new(Value::Integer(1));
        assert_eq!(
            Parser::new("(1)").read(),
            Some(Ok(Value::Gene(result)))
        );
    }
    {
        let mut result = Gene::new(Value::Integer(1));
        result.data.push(Box::new(Value::Integer(2)));
        assert_eq!(
            Parser::new("(1 2)").read(),
            Some(Ok(Value::Gene(result)))
        );
    }
    {
        let mut result = Gene::new(Value::Integer(1));
        result.props.insert("key".into(), Box::new(Value::Integer(2)));
        result.data.push(Box::new(Value::Integer(3)));
        assert_eq!(
            Parser::new("(1 ^key 2 3)").read(),
            Some(Ok(Value::Gene(result)))
        );
    }
    {
        let mut result = Gene::new(Value::Integer(1));
        result.data.push(Box::new(Value::Array(Vec::new())));
        assert_eq!(
            Parser::new("(1 [])").read(),
            Some(Ok(Value::Gene(result)))
        );
    }
    {
        let mut result = Gene::new(Value::Integer(1));
        result.props.insert("key".into(), Box::new(Value::Integer(123)));
        result.data.push(Box::new(Value::Array(Vec::new())));
        assert_eq!(
            Parser::new("(1 ^key 123 [])").read(),
            Some(Ok(Value::Gene(result)))
        );
    }
}

#[test]
fn test_quote() {
    {
        let mut result = Gene::new(Value::Symbol("#QUOTE".into()));
        result.data.push(Box::new(Value::Symbol("ab".into())));
        assert_eq!(
            Parser::new("`ab").read(),
            Some(Ok(Value::Gene(result)))
        );
    }
    {
        let mut result = Gene::new(Value::Symbol("#QUOTE".into()));
        result.data.push(Box::new(Value::Boolean(true)));
        assert_eq!(
            Parser::new("`true").read(),
            Some(Ok(Value::Gene(result)))
        );
    }
    {
        let mut result = Gene::new(Value::Symbol("#QUOTE".into()));
        result.data.push(Box::new(Value::Array(Vec::new())));
        assert_eq!(
            Parser::new("`[]").read(),
            Some(Ok(Value::Gene(result)))
        );
    }
}