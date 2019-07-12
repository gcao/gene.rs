extern crate gene;

use std::collections::*;
use std::time::*;

use gene::compiler::Compiler;
use gene::parser::Parser;
use gene::types::Value;
use gene::vm::VirtualMachine;

struct Dummy {
    pos: usize,
    total_time: Duration,
    recent_start_time: Instant,
    arr: [i64; 16],
    vec: Vec<i64>,
    map: HashMap<usize, i64>,
    map2: BTreeMap<usize, i64>,
}

impl Dummy {
    pub fn new() -> Self {
        Dummy {
            pos: 0,
            total_time: Duration::new(0, 0),
            recent_start_time: Instant::now(),
            arr: [0; 16],
            vec: Vec::new(),
            map: HashMap::new(),
            map2: BTreeMap::new(),
        }
    }

    pub fn report_start(&mut self) {
        self.recent_start_time = Instant::now()
    }

    pub fn report_end(&mut self) {
        self.total_time += self.recent_start_time.elapsed();
    }

    pub fn calibrate_perf(&mut self) {
        let mut result = 0;

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
        show("Increment struct property", time.as_nanos());

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
        show("Increment local variable", time.as_nanos());

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
        show("Report_start/report_end", self.total_time.as_nanos());

        let start = Instant::now();
        self.arr[0] = 1;
        self.arr[1] = 1;
        self.arr[2] = 1;
        self.arr[3] = 1;
        self.arr[4] = 1;
        self.arr[5] = 1;
        self.arr[6] = 1;
        self.arr[7] = 1;
        self.arr[8] = 1;
        self.arr[9] = 1;
        let time = start.elapsed();
        show("Array write", time.as_nanos());

        let start = Instant::now();
        result += self.arr[0];
        result += self.arr[1];
        result += self.arr[2];
        result += self.arr[3];
        result += self.arr[4];
        result += self.arr[5];
        result += self.arr[6];
        result += self.arr[7];
        result += self.arr[8];
        result += self.arr[9];
        let time = start.elapsed();
        show("Array read", time.as_nanos());

        let start = Instant::now();
        self.vec.insert(0, 1);
        self.vec.insert(0, 1);
        self.vec.insert(0, 1);
        self.vec.insert(0, 1);
        self.vec.insert(0, 1);
        self.vec.insert(0, 1);
        self.vec.insert(0, 1);
        self.vec.insert(0, 1);
        self.vec.insert(0, 1);
        self.vec.insert(0, 1);
        let time = start.elapsed();
        show("Vec insert", time.as_nanos());

        let start = Instant::now();
        result += self.vec[0];
        result += self.vec[1];
        result += self.vec[2];
        result += self.vec[3];
        result += self.vec[4];
        result += self.vec[5];
        result += self.vec[6];
        result += self.vec[7];
        result += self.vec[8];
        result += self.vec[9];
        let time = start.elapsed();
        show("Vec read", time.as_nanos());

        let start = Instant::now();
        self.map.insert(0, 1);
        self.map.insert(1, 1);
        self.map.insert(2, 1);
        self.map.insert(3, 1);
        self.map.insert(4, 1);
        self.map.insert(5, 1);
        self.map.insert(6, 1);
        self.map.insert(7, 1);
        self.map.insert(8, 1);
        self.map.insert(9, 1);
        let time = start.elapsed();
        show("HashMap insert", time.as_nanos());

        let start = Instant::now();
        result += self.map[&0];
        result += self.map[&1];
        result += self.map[&2];
        result += self.map[&3];
        result += self.map[&4];
        result += self.map[&5];
        result += self.map[&6];
        result += self.map[&7];
        result += self.map[&8];
        result += self.map[&9];
        let time = start.elapsed();
        show("HashMap read", time.as_nanos());

        let start = Instant::now();
        self.map2.insert(0, 1);
        self.map2.insert(1, 1);
        self.map2.insert(2, 1);
        self.map2.insert(3, 1);
        self.map2.insert(4, 1);
        self.map2.insert(5, 1);
        self.map2.insert(6, 1);
        self.map2.insert(7, 1);
        self.map2.insert(8, 1);
        self.map2.insert(9, 1);
        let time = start.elapsed();
        show("BTreeMap insert", time.as_nanos());

        let start = Instant::now();
        result += self.map2[&0];
        result += self.map2[&1];
        result += self.map2[&2];
        result += self.map2[&3];
        result += self.map2[&4];
        result += self.map2[&5];
        result += self.map2[&6];
        result += self.map2[&7];
        result += self.map2[&8];
        result += self.map2[&9];
        let time = start.elapsed();
        show("BTreeMap read", time.as_nanos());

        println!("IGNORE THIS: {}\n", result);
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

fn show(name: &str, time: u128) {
    println!("{:>40}: {:9.3} ns", name, time as f64 / 10.);
}