use super::{Cpu, Memory, Pids};

/// Control groups resources builder.
#[derive(Clone, Default, Debug)]
pub struct Resources {
    cpu: Option<Cpu>,
    memory: Option<Memory>,
    pids: Option<Pids>,
}

impl Resources {
    /// Sets the cgroup cpu.
    pub fn cpu(&mut self, cpu: Cpu) -> &mut Self {
        self.cpu = Some(cpu);
        self
    }

    /// Sets the cgroup memory.
    pub fn memory(&mut self, memory: Memory) -> &mut Self {
        self.memory = Some(memory);
        self
    }

    /// Sets the cgroup pids.
    pub fn pids(&mut self, pids: Pids) -> &mut Self {
        self.pids = Some(pids);
        self
    }
}
