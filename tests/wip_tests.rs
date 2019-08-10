// This file is used for debugging specific test
// It is referenced in .vscode/tasks.json like
// cargo test --no-run --message-format=json temp_tests

#![allow(unused_imports)]

extern crate ordered_float;
extern crate gene;

use ordered_float::OrderedFloat;
use std::collections::HashMap;

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
            (var a 0)
            (while (a < 2)
                (a = (a + 1))
            )
            a
        ");
        let parsed = parser.parse();
        let module_temp = compiler.compile(parsed.unwrap());
        let module = &module_temp.borrow();
        let result_temp = vm.load_module(module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(2));
    }
}
