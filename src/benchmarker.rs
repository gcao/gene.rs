use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Benchmarker {
    loop_count: usize,
    loop_start_time: Instant,
    total_time: Duration,
    op_times: BTreeMap<String, OpTime>,
    last_op: String,
}

impl Benchmarker {
    pub fn new() -> Self {
        Benchmarker {
            loop_count: 0,
            loop_start_time: Instant::now(),
            total_time: Duration::new(0, 0),
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
        self.total_time = self.loop_start_time.elapsed();
        for (_name, op_time) in self.op_times.iter_mut() {
            op_time.calc_percentage(self.total_time.as_nanos() as f64);
        }
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

    pub fn loop_time(&self) -> Duration {
        self.total_time - self.op_times.values().map(|item| item.total_time ).sum()
    }

    pub fn loop_average_time(&self) -> f64 {
        self.loop_time().as_nanos() as f64 / self.loop_count as f64
    }
}

impl fmt::Display for Benchmarker {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("<<< BENCHMARK BEGIN >>>\n\n")?;
        fmt.write_str(&*format!("{: >20}: {:7.3}% {:13.8}\n",
          "Total",
          100.,
          self.total_time.as_nanos() as f64 / 1_000_000_000.))?;

        fmt.write_str(&*format!("{: >20}: {:7.3}% {:13.8} / {:8} = {:8.0} ns\n",
          "Loop",
          self.loop_time().as_nanos() as f64 * 100. / self.total_time.as_nanos() as f64,
          self.loop_time().as_nanos() as f64 / 1_000_000_000.,
          self.loop_count,
          self.loop_average_time()))?;

        let mut sorted_op_times: Vec<OpTime> = self.op_times.values().cloned().collect();
        sorted_op_times.sort_by(|first, second| second.total_time.cmp(&first.total_time));
        for op_time in sorted_op_times {
            fmt.write_str(&*format!("{}", op_time))?;
        }

        fmt.write_str("\n<<<  BENCHMARK END  >>>\n\n")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct OpTime {
    name: String,
    count: usize,
    total_time: Duration,
    recent_start_time: Instant,
    percentage: f64,
}

impl OpTime {
    pub fn new(name: String) -> Self {
        OpTime {
            name,
            count: 0,
            total_time: Duration::new(0, 0),
            recent_start_time: Instant::now(),
            percentage: 0.,
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
        self.total_time.as_nanos() as f64 / self.count as f64
    }

    pub fn calc_percentage(&mut self, total: f64) {
        self.percentage = self.total_time.as_nanos() as f64 / total;
    }
}

impl fmt::Display for OpTime {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&*format!("{: >20}: {:7.3}% {:13.8} / {:8} = {:8.0} ns\n",
          self.name,
          self.percentage * 100.,
          self.total_time.as_nanos() as f64 / 1_000_000_000.,
          self.count,
          self.average_time()))?;
        Ok(())
    }
}
