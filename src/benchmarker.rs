use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Benchmarker {
    loop_count: usize,
    loop_start_time: Instant,
    total_time: Duration,
    pub init_time: OpTime,
    pub default_time: OpTime,
    pub save_time: OpTime,
    pub copy_from_default_time: OpTime,
    pub copy_to_default_time: OpTime,
    pub def_member_time: OpTime,
    pub get_member_time: OpTime,
    pub set_member_time: OpTime,
    pub function_time: OpTime,
    pub create_arguments_time: OpTime,
    pub call_time: OpTime,
    pub call_end_time: OpTime,
    pub jump_time: OpTime,
    pub jump_if_false_time: OpTime,
    pub get_item_time: OpTime,
    pub set_item_time: OpTime,
    pub binary_op_time: OpTime,
}

impl Benchmarker {
    pub fn new() -> Self {
        Benchmarker {
            loop_count: 0,
            loop_start_time: Instant::now(),
            total_time: Duration::new(0, 0),
            init_time: OpTime::new("Init".to_string()),
            default_time: OpTime::new("Default".to_string()),
            save_time: OpTime::new("Save".to_string()),
            copy_from_default_time: OpTime::new("CopyFromDefault".to_string()),
            copy_to_default_time: OpTime::new("CopyToDefault".to_string()),
            def_member_time: OpTime::new("DefMember".to_string()),
            get_member_time: OpTime::new("GetMember".to_string()),
            set_member_time: OpTime::new("SetMember".to_string()),
            function_time: OpTime::new("Function".to_string()),
            create_arguments_time: OpTime::new("CreateArguments".to_string()),
            call_time: OpTime::new("Call".to_string()),
            call_end_time: OpTime::new("CallEnd".to_string()),
            jump_time: OpTime::new("Jump".to_string()),
            jump_if_false_time: OpTime::new("JumpIfFalse".to_string()),
            get_item_time: OpTime::new("GetItem".to_string()),
            set_item_time: OpTime::new("SetItem".to_string()),
            binary_op_time: OpTime::new("BinaryOp".to_string()),
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
    }

    pub fn op_times(&self) -> Vec<OpTime> {
        let mut op_times = Vec::new();
        op_times.push(self.init_time.clone());
        op_times.push(self.default_time.clone());
        op_times.push(self.save_time.clone());
        op_times.push(self.copy_from_default_time.clone());
        op_times.push(self.copy_to_default_time.clone());
        op_times.push(self.def_member_time.clone());
        op_times.push(self.set_member_time.clone());
        op_times.push(self.get_member_time.clone());
        op_times.push(self.function_time.clone());
        op_times.push(self.create_arguments_time.clone());
        op_times.push(self.call_time.clone());
        op_times.push(self.call_end_time.clone());
        op_times.push(self.jump_time.clone());
        op_times.push(self.jump_if_false_time.clone());
        op_times.push(self.get_item_time.clone());
        op_times.push(self.set_item_time.clone());
        op_times.push(self.binary_op_time.clone());
        op_times
    }

    pub fn loop_time(&self) -> Duration {
        self.total_time - self.op_times().iter().map(|item| item.total_time ).sum()
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

        let mut op_times = self.op_times();
        op_times.sort_by(|first, second| second.total_time.cmp(&first.total_time));
        for op_time in op_times {
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
