/// Represents the cgroup subsystems cpu.
#[derive(Clone, Debug)]
pub struct Cpu {
    shares: Option<u64>,
    period: Option<u64>,
    quota: Option<u64>,
}

impl Cpu {
    /// Specifies a relative share of CPU time available to the tasks in a cgroup.
    pub fn shares(&mut self, val: u64) -> &mut Self {
        self.shares = Some(val);
        self
    }

    /// Specifies a period of time in microseconds for how regularly a cgroup's
    /// access to CPU resources should be reallocated (CFS scheduler only).
    pub fn period(&mut self, val: u64) -> &mut Self {
        self.period = Some(val);
        self
    }

    /// Specifies the total amount of time in microseconds for which all tasks
    /// in a cgroup can run during one period (as defined by period).
    pub fn quota(&mut self, val: u64) -> &mut Self {
        self.quota = Some(val);
        self
    }
}
