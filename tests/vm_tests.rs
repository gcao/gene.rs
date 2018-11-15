extern crate gene;

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
        let result = (*vm.load_module(module)).downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Integer(1));
    }
    {
        let mut parser = Parser::new("\"ab\"");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let result = (*vm.load_module(module)).downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::String("ab".into()));
    }
    {
        let mut parser = Parser::new("true");
        let parsed = parser.parse();
        let module = compiler.compile(parsed.unwrap());
        let result = (*vm.load_module(module)).downcast_ref::<Value>().unwrap();
        assert_eq!(*result, Value::Boolean(true));
    }
}