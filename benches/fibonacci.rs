#[macro_use]
extern crate criterion;
extern crate gene;

use criterion::Criterion;
use criterion::black_box;

use gene::compiler2::Compiler;
use gene::parser::Parser;
use gene::types::Value;
use gene::vm::VirtualMachine;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut parser = Parser::new("
        (fn fibonacci n
            (if (n < 2)
                n
            else
                ((fibonacci (n - 1)) + (fibonacci (n - 2)))
            )
        )
        (fibonacci 20)
    ");
    let parsed = parser.parse();
    let mut compiler = Compiler::new();
    compiler.compile(parsed.unwrap());
    let module = compiler.module;
    let mut vm = VirtualMachine::new();

    c.bench_function("fib 20", |b| b.iter(||
        vm.load_module(&module)
    ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
