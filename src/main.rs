extern crate gene;

use std::time::*;

use gene::compiler::Compiler;
use gene::parser::Parser;
use gene::types::Value;
use gene::vm::VirtualMachine;

struct Dummy {
    pos: usize,
    total_time: Duration,
    recent_start_time: Instant,
}

impl Dummy {
    pub fn new() -> Self {
        Dummy {
            pos: 0,
            total_time: Duration::new(0, 0),
            recent_start_time: Instant::now(),
        }
    }

    pub fn report_start(&mut self) {
        self.recent_start_time = Instant::now()
    }

    pub fn report_end(&mut self) {
        self.total_time += self.recent_start_time.elapsed();
    }

    pub fn calibrate_perf(&mut self) {
        let start = Instant::now();
        self.pos += 1;
        self.pos += 2;
        self.pos += 3;
        self.pos += 4;
        self.pos += 5;
        self.pos += 6;
        self.pos += 7;
        self.pos += 8;
        self.pos += 9;
        self.pos += 10;
        let time = start.elapsed();
        println!("Increment struct property: {:6.3} ns", time.as_nanos() as f64 / 10.);

        let mut _pos2 = 0;
        let start2 = Instant::now();
        _pos2 += 1;
        _pos2 += 2;
        _pos2 += 3;
        _pos2 += 4;
        _pos2 += 5;
        _pos2 += 6;
        _pos2 += 7;
        _pos2 += 8;
        _pos2 += 9;
        _pos2 += 10;
        let time2 = start2.elapsed();
        println!("Increment local variable: {:6.3} ns", time2.as_nanos() as f64 / 10.);

        self.report_start();
        self.pos += 1;
        self.report_end();
        self.report_start();
        self.pos += 2;
        self.report_end();
        self.report_start();
        self.pos += 3;
        self.report_end();
        self.report_start();
        self.pos += 4;
        self.report_end();
        self.report_start();
        self.pos += 5;
        self.report_end();
        self.report_start();
        self.pos += 6;
        self.report_end();
        self.report_start();
        self.pos += 7;
        self.report_end();
        self.report_start();
        self.pos += 8;
        self.report_end();
        self.report_start();
        self.pos += 9;
        self.report_end();
        self.report_start();
        self.pos += 10;
        self.report_end();
        println!("Report_start/report_end: {:6.3} ns\n", self.total_time.as_nanos() as f64 / 10.);
    }
}

fn main() {
    let mut dummy = Dummy::new();
    dummy.calibrate_perf();

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
