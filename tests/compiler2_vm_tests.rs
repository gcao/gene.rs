#[macro_use]
extern crate gene;

use std::collections::HashMap;

use ordered_float::OrderedFloat;

use gene::compiler2::Compiler;
use gene::parser::Parser;
use gene::types::Value;
use gene::vm::VirtualMachine;

#[test]
fn test_basic_stmts() {
    {
        let mut parser = Parser::new("1");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
    {
        let mut parser = Parser::new("\"ab\"");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::String("ab".to_string()));
    }
    {
        let mut parser = Parser::new("[]");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(Vec::new()));
    }
    {
        let mut parser = Parser::new("[1]");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(vec![Value::Integer(1)]));
    }
    {
        let mut parser = Parser::new("{}");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Map(HashMap::new()));
    }
    {
        let mut parser = Parser::new("[1]");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(vec![Value::Integer(1)]));
    }
    {
        let mut parser = Parser::new("{^key 1}");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(
            *result,
            Value::Map(map! {
                "key" => Value::Integer(1),
            })
        );
    }
}

#[test]
fn test_variables() {
    {
        let mut parser = Parser::new("
            # Define variable <a>
            (var a 1)
            # Return <a>'s value
            a
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
    {
        let mut parser = Parser::new("
            (var a 1)
            (var b 3)
            [a 2 b]
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
    }
    {
        let mut parser = Parser::new("
            (var a 1)
            (var b 2)
            [a b (a + b)]
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
    }
}

#[test]
fn test_binary_operations() {
    {
        let mut parser = Parser::new("
            (1 + 2)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(3));
    }
    {
        let mut parser = Parser::new("
            (1 < 2)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Boolean(true));
    }
}

#[test]
fn test_assignments() {
    {
        let mut parser = Parser::new("
            (var a 1)
            (a = 2)
            a
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(2));
    }
}

#[test]
fn test_ifs() {
    {
        let mut parser = Parser::new("
            (if true 1 else 2)
        ");
        let parsed = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        dbg!(module.get_default_block());
        let result_temp = VirtualMachine::new().load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
}
