/// Represents the cgroup subsystem memory.
#[derive(Clone, Default, Debug)]
pub struct Memory {
    limit: Option<i64>,
    reservation: Option<i64>,
    swap: Option<i64>,
}

impl Memory {
    /// Sets limit of memory usage.
    pub fn limit(&mut self, val: i64) -> &mut Self {
        self.limit = Some(val);
        self
    }

    /// Sets soft limit of memory usage.
    pub fn reservation(&mut self, val: i64) -> &mut Self {
        self.reservation = Some(val);
        self
    }

    /// Sets limit of memory+swap usage.
    pub fn swap(&mut self, val: i64) -> &mut Self {
        self.swap = Some(val);
        self
    }
}
