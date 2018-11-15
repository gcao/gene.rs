// This file is used for debugging specific test
// It is referenced in .vscode/tasks.json like
// cargo test --no-run --message-format=json temp_tests

extern crate ordered_float;
extern crate gene;

use ordered_float::OrderedFloat;
use std::collections::{BTreeMap};

use gene::parser::Parser;
use gene::types::Value;
use gene::types::Gene;
use gene::compiler::Compiler;
use gene::vm::VirtualMachine;

#[test]
fn test_this() {
    let mut compiler = Compiler::new();
    let mut vm = VirtualMachine::new();
    {
        let mut parser = Parser::new("true");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let result = (*vm.load_module(module)).downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Boolean(true));
    }
}
