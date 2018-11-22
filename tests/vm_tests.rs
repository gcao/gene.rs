#[macro_use] extern crate gene;

use std::collections::{BTreeMap};

use ordered_float::OrderedFloat;

use gene::types::Value;
use gene::parser::Parser;
use gene::compiler::Compiler;
use gene::vm::VirtualMachine;

#[test]
fn test_basic_stmts() {
    let mut compiler = Compiler::new();
    let mut vm = VirtualMachine::new();
    {
        let mut parser = Parser::new("1");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
    {
        let mut parser = Parser::new("1.1");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Float(OrderedFloat(1.1)));
    }
    {
        let mut parser = Parser::new("\"ab\"");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::String("ab".to_string()));
    }
    {
        let mut parser = Parser::new("null");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Null);
    }
    {
        let mut parser = Parser::new("true");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Boolean(true));
    }
    {
        let mut parser = Parser::new("[]");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(Vec::new()));
    }
    {
        let mut parser = Parser::new("[1]");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Array(vec![Value::Integer(1)]));
    }
    {
        let mut parser = Parser::new("{}");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Map(BTreeMap::new()));
    }
    {
        let mut parser = Parser::new("{^key 1}");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Map(map!{
            "key" => Value::Integer(1),
        }));
    }
}

#[test]
fn test_variables() {
    let mut compiler = Compiler::new();
    let mut vm = VirtualMachine::new();
    {
        let mut parser = Parser::new("
            # Define variable <a>
            (var a 1)
            # Return <a>'s value
            a
        ");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
}

#[test]
fn test_binary_operations() {
    let mut compiler = Compiler::new();
    let mut vm = VirtualMachine::new();
    {
        let mut parser = Parser::new("
          (1 + 2)
        ");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(3));
    }
}

#[test]
fn test_functions() {
    let mut compiler = Compiler::new();
    let mut vm = VirtualMachine::new();
    {
        // TODO
    }
}