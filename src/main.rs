extern crate gene;

use gene::compiler::Compiler;
use gene::parser::Parser;
use gene::types::Value;
use gene::vm::VirtualMachine;

fn main() {
    let mut compiler = Compiler::new();
    let mut vm = VirtualMachine::new();

    let mut parser = Parser::new("
      (fn fibonacci n
        (if (n < 2)
          n
        else
          ((fibonacci (n - 1)) + (fibonacci (n - 2)))
        )
      )
      (fibonacci 24)
    ");
    let parsed = parser.parse();
    let module_temp = compiler.compile(parsed.unwrap());
    let module = &module_temp.borrow();
    let result_temp = vm.load_module(module);
    let borrowed = result_temp.borrow();
    let result = borrowed.downcast_ref::<Value>().unwrap();
    println!("Result: {}", result);
}
