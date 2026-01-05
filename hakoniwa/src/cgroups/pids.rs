/// Represents the cgroup subsystem pids.
#[derive(Clone, Default, Debug)]
pub struct Pids {
    limit: Option<i64>,
}

impl Pids {
    /// Specifies the maximum number of tasks in the cgroup.
    pub fn limit(&mut self, val: i64) -> &mut Self {
        self.limit = Some(val);
        self
    }
}
