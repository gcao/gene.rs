extern crate gene;

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::time::*;

use gene::compiler::Compiler;
use gene::parser::Parser;
use gene::types::Value;
use gene::vm::VirtualMachine;

struct Dummy {
    pos: usize,
    total_time: Duration,
    recent_start_time: Instant,
    arr: [Rc<RefCell<String>>; 10],
    map: HashMap<usize, Rc<RefCell<String>>>,
}

impl Dummy {
    pub fn new() -> Self {
        let s = Rc::new(RefCell::new("test".to_string()));
        Dummy {
            pos: 0,
            total_time: Duration::new(0, 0),
            recent_start_time: Instant::now(),
            arr: [s.clone(), s.clone(), s.clone(), s.clone(), s.clone(), s.clone(), s.clone(), s.clone(), s.clone(), s.clone()],
            map: HashMap::new(),
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

        let mut _pos = 0;
        let start = Instant::now();
        _pos += 1;
        _pos += 2;
        _pos += 3;
        _pos += 4;
        _pos += 5;
        _pos += 6;
        _pos += 7;
        _pos += 8;
        _pos += 9;
        _pos += 10;
        let time = start.elapsed();
        println!("Increment local variable: {:6.3} ns", time.as_nanos() as f64 / 10.);

        // 1
        self.report_start();
        self.report_end();
        // 2
        self.report_start();
        self.report_end();
        // 3
        self.report_start();
        self.report_end();
        // 4
        self.report_start();
        self.report_end();
        // 5
        self.report_start();
        self.report_end();
        // 6
        self.report_start();
        self.report_end();
        // 7
        self.report_start();
        self.report_end();
        // 8
        self.report_start();
        self.report_end();
        // 9
        self.report_start();
        self.report_end();
        // 10
        self.report_start();
        self.report_end();
        println!("Report_start/report_end: {:6.3} ns", self.total_time.as_nanos() as f64 / 10.);

        let s = Rc::new(RefCell::new("s".to_string()));
        let start = Instant::now();
        self.arr[0] = s.clone();
        self.arr[1] = s.clone();
        self.arr[2] = s.clone();
        self.arr[3] = s.clone();
        self.arr[4] = s.clone();
        self.arr[5] = s.clone();
        self.arr[6] = s.clone();
        self.arr[7] = s.clone();
        self.arr[8] = s.clone();
        self.arr[9] = s.clone();
        let time = start.elapsed();
        println!("Access array: {:6.3} ns", time.as_nanos() as f64 / 10.);

        let start = Instant::now();
        self.map.insert(0, s.clone());
        self.map.insert(1, s.clone());
        self.map.insert(2, s.clone());
        self.map.insert(3, s.clone());
        self.map.insert(4, s.clone());
        self.map.insert(5, s.clone());
        self.map.insert(6, s.clone());
        self.map.insert(7, s.clone());
        self.map.insert(8, s.clone());
        self.map.insert(9, s.clone());
        let time = start.elapsed();
        println!("Access map: {:6.3} ns", time.as_nanos() as f64 / 10.);

        println!("");
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
