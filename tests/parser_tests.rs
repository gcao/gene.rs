#[macro_use]
extern crate gene;

use std::collections::HashMap;

use ordered_float::OrderedFloat;

use gene::parser::Parser;
use gene::types::Gene;
use gene::types::Value;

#[test]
fn test_read_empty_input() {
    assert_eq!(Parser::new("").read(), None);
    assert_eq!(Parser::new("  ").read(), None);
}

#[test]
fn test_read_number() {
    assert_eq!(Parser::new("1").read(), Some(Ok(Value::Integer(1))));
    assert_eq!(Parser::new("+1").read(), Some(Ok(Value::Integer(1))));
    assert_eq!(Parser::new("-1").read(), Some(Ok(Value::Integer(-1))));

    assert_eq!(
        Parser::new("1.1").read(),
        Some(Ok(Value::Float(OrderedFloat(1.1))))
    );
    assert_eq!(
        Parser::new("-1.1").read(),
        Some(Ok(Value::Float(OrderedFloat(-1.1))))
    );
}

// read_word() is not a public method, should not be tested directly
// If it has to be tested, parser.next() should be called first.
// #[test]
// fn test_read_word() {
//     assert_eq!(Parser::new("ab").read_word(), Some(Ok("ab".to_string())));
//     assert_eq!(Parser::new("ab cd").read_word(), Some(Ok("ab".to_string())));
//     assert_eq!(Parser::new("ab,cd").read_word(), Some(Ok("ab".to_string())));
//     assert_eq!(Parser::new("你好").read_word(), Some(Ok("你好".to_string())));
// }

#[test]
fn test_read_keywords() {
    assert_eq!(Parser::new("true").read(), Some(Ok(Value::Boolean(true))));
    assert_eq!(Parser::new(" true ").read(), Some(Ok(Value::Boolean(true))));
    assert_eq!(Parser::new("false").read(), Some(Ok(Value::Boolean(false))));
    assert_eq!(Parser::new("null").read(), Some(Ok(Value::Null)));
    assert_eq!(
        Parser::new("\\true").read(),
        Some(Ok(Value::Symbol("true".to_string())))
    );
}

#[test]
fn test_read_string() {
    assert_eq!(
        Parser::new("\"ab\"").read(),
        Some(Ok(Value::String("ab".to_string())))
    );
    assert_eq!(
        Parser::new("\"a\nb\"").read(),
        Some(Ok(Value::String("a\nb".to_string())))
    );
    assert_eq!(
        Parser::new("\"ab \\\"cd\\\"\"").read(),
        Some(Ok(Value::String("ab \"cd\"".to_string())))
    );
    assert_eq!(
        Parser::new("\"你好\"").read(),
        Some(Ok(Value::String("你好".to_string())))
    );
}

#[test]
fn test_skip_comment() {
    assert_eq!(
        Parser::new("#\nab").read(),
        Some(Ok(Value::Symbol("ab".to_string())))
    );
    assert_eq!(
        Parser::new("#!test\nab").read(),
        Some(Ok(Value::Symbol("ab".to_string())))
    );
}

#[test]
fn test_read_symbols() {
    assert_eq!(
        Parser::new("ab").read(),
        Some(Ok(Value::Symbol("ab".to_string())))
    );
    assert_eq!(
        Parser::new("你好").read(),
        Some(Ok(Value::Symbol("你好".to_string())))
    );
    {
        let mut parser = Parser::new("ab");
        assert_eq!(parser.read(), Some(Ok(Value::Symbol("ab".to_string()))));
        assert_eq!(parser.read(), None);
    }
}

#[test]
fn test_read_array() {
    assert_eq!(Parser::new("[]").read(), Some(Ok(Value::Array(Vec::new()))));
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
    assert_eq!(
        Parser::new("{}").read(),
        Some(Ok(Value::Map(HashMap::new())))
    );
    assert_eq!(
        Parser::new("{^key 123}").read(),
        Some(Ok(Value::Map(map! {
            "key" => Value::Integer(123),
        })))
    );
    assert_eq!(
        Parser::new("{^^key}").read(),
        Some(Ok(Value::Map(map! {
            "key" => Value::Boolean(true),
        })))
    );
    assert_eq!(
        Parser::new("{^!key}").read(),
        Some(Ok(Value::Map(map! {
            "key" => Value::Boolean(false),
        })))
    );
    assert_eq!(
        Parser::new("{^key [123]}").read(),
        Some(Ok(Value::Map(map! {
            "key" => Value::Array(vec![Value::Integer(123)]),
        })))
    );
}

#[test]
fn test_read_gene() {
    assert_eq!(
        Parser::new("()").read(),
        Some(Ok(Value::Gene(Box::new(Gene::new(Value::Void)))))
    );
    {
        let result = Gene::new(Value::Integer(1));
        assert_eq!(Parser::new("(1)").read(), Some(Ok(Value::Gene(Box::new(result)))));
    }
    {
        let mut result = Gene::new(Value::Integer(1));
        result.data.push(Value::Integer(2));
        assert_eq!(Parser::new("(1 2)").read(), Some(Ok(Value::Gene(Box::new(result)))));
    }
    {
        let mut result = Gene::new(Value::Integer(1));
        result.props.insert("key".to_string(), Value::Integer(2));
        result.data.push(Value::Integer(3));
        assert_eq!(
            Parser::new("(1 ^key 2 3)").read(),
            Some(Ok(Value::Gene(Box::new(result))))
        );
    }
    {
        let mut result = Gene::new(Value::Integer(1));
        result.data.push(Value::Array(Vec::new()));
        assert_eq!(Parser::new("(1 [])").read(), Some(Ok(Value::Gene(Box::new(result)))));
    }
    {
        let mut result = Gene::new(Value::Integer(1));
        result.props.insert("key".to_string(), Value::Integer(123));
        result.data.push(Value::Array(Vec::new()));
        assert_eq!(
            Parser::new("(1 ^key 123 [])").read(),
            Some(Ok(Value::Gene(Box::new(result))))
        );
    }
}

#[test]
fn test_quote() {
    {
        let mut result = Gene::new(Value::Symbol("#QUOTE".to_string()));
        result.data.push(Value::Symbol("ab".to_string()));
        assert_eq!(Parser::new("`ab").read(), Some(Ok(Value::Gene(Box::new(result)))));
    }
    {
        let mut result = Gene::new(Value::Symbol("#QUOTE".to_string()));
        result.data.push(Value::Boolean(true));
        assert_eq!(Parser::new("`true").read(), Some(Ok(Value::Gene(Box::new(result)))));
    }
    {
        let mut result = Gene::new(Value::Symbol("#QUOTE".to_string()));
        result.data.push(Value::Array(Vec::new()));
        assert_eq!(Parser::new("`[]").read(), Some(Ok(Value::Gene(Box::new(result)))));
    }
}

#[test]
fn test_parse_one_string() {
    assert_eq!(
        Parser::new("\"ab\"").parse(),
        Ok(Value::String("ab".to_string()))
    );
}

#[test]
fn test_parse_stream() {
    {
        let result = Value::Stream(vec![
            Value::Symbol("ab".to_string()),
            Value::Symbol("cd".to_string()),
        ]);
        assert_eq!(Parser::new("ab cd").parse(), Ok(result));
    }
}
