use std::collections::BTreeMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Benchmarker {
    loop_count: usize,
    loop_time: Duration,
    loop_start_time: Instant,
    op_times: BTreeMap<String, OpTime>,
    last_op: String,
}

impl Benchmarker {
    pub fn new() -> Self {
        Benchmarker {
            loop_count: 0,
            loop_time: Duration::new(0, 0),
            loop_start_time: Instant::now(),
            op_times: BTreeMap::new(),
            last_op: "".to_string(),
        }
    }

    pub fn report_loop(&mut self) {
        self.loop_count += 1;
    }

    pub fn loop_start(&mut self) {
        self.loop_start_time = Instant::now();
    }

    pub fn loop_end(&mut self) {
        self.loop_time = self.loop_start_time.elapsed();
    }

    pub fn op_start(&mut self, name: &str) {
        self.last_op = name.to_string();
        if let Some(op_time) = self.op_times.get_mut(name) {
            op_time.report_start();
        } else {
            let op_time = OpTime::new(name.to_string());
            self.op_times.insert(name.to_string(), op_time);
        }
    }

    pub fn op_end(&mut self) {
        self.op_times.get_mut(&self.last_op).unwrap().report_end();
    }
}

#[derive(Debug)]
pub struct OpTime {
    name: String,
    count: usize,
    total_time: Duration,
    recent_start_time: Instant,
}

impl OpTime {
    pub fn new(name: String) -> Self {
        OpTime {
            name,
            count: 0,
            total_time: Duration::new(0, 0),
            recent_start_time: Instant::now(),
        }
    }

    pub fn report_start(&mut self) {
        self.recent_start_time = Instant::now()
    }

    pub fn report_partial(&mut self) {
        self.total_time += self.recent_start_time.elapsed();
    }

    pub fn report_end(&mut self) {
        self.count += 1;
        self.total_time += self.recent_start_time.elapsed();
    }

    pub fn average_time(&self) -> f64 {
        (self.total_time.as_nanos() as f64) / (self.count as f64)
    }
}
