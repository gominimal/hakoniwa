use super::error::*;

/// Represents the cgroup subsystems cpu.
#[derive(Clone, Default, Debug)]
pub struct Cpu {
    shares: Option<u64>,
    period: Option<u64>,
    quota: Option<i64>,
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
    pub fn quota(&mut self, val: i64) -> &mut Self {
        self.quota = Some(val);
        self
    }

    /// Build.
    pub(crate) fn build(&self) -> Result<oci_spec::runtime::LinuxCpu> {
        let mut builder = oci_spec::runtime::LinuxCpuBuilder::default();
        if let Some(val) = self.shares {
            builder = builder.shares(val);
        }
        if let Some(val) = self.period {
            builder = builder.period(val);
        }
        if let Some(val) = self.quota {
            builder = builder.quota(val);
        }
        Ok(builder.build()?)
    }
}
