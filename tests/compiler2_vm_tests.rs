#[macro_use]
extern crate gene;

use std::collections::BTreeMap;

use ordered_float::OrderedFloat;

use gene::compiler2::Compiler;
use gene::parser::Parser;
use gene::types::Value;
use gene::vm::VirtualMachine;

#[test]
fn test_basic_stmts() {
    let mut compiler = Compiler::new();
    let mut vm = VirtualMachine::new();
    {
        let mut parser = Parser::new("1");
        let parsed = parser.parse();
        compiler.compile(parsed.unwrap());
        let module = compiler.module;
        println!("{}", module.get_default_block());
        let result_temp = vm.load_module(&module);
        let borrowed = result_temp.borrow();
        let result = borrowed.downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
}
