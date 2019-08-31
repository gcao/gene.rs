// This file is used for debugging specific test
// It is referenced in .vscode/tasks.json like
// cargo test --no-run --message-format=json temp_tests

#![allow(unused_imports)]

extern crate ordered_float;
extern crate gene;

use ordered_float::OrderedFloat;
use std::collections::HashMap;

use gene::compiler2::Compiler;
use gene::parser::Parser;
use gene::types::Gene;
use gene::types::Value;
use gene::vm::VirtualMachine;

#[cfg(feature = "wip_tests")]
#[test]
fn test_wip() {
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
