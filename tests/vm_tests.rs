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
        let result = vm.process(module);
        assert_eq!(result, Value::Integer(1));
    }
}