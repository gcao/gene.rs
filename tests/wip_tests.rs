// This file is used for debugging specific test
// It is referenced in .vscode/tasks.json like
// cargo test --no-run --message-format=json temp_tests

extern crate ordered_float;
extern crate gene;

use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

use gene::compiler::Compiler;
use gene::parser::Parser;
use gene::types::Gene;
use gene::types::Value;
use gene::vm::VirtualMachine;

#[cfg(feature = "wip_tests")]
#[test]
fn test_wip() {
    let mut compiler = Compiler::new();
    let mut vm = VirtualMachine::new();
    {
        let mut parser = Parser::new("
            (fn f _ 1)
            (f)
        ");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let borrowed = (*vm.load_module(module)).borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
}
