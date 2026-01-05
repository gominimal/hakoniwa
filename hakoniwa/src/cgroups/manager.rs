use super::{Resources, error::*};

#[derive(Clone, Debug)]
pub(crate) struct Manager {}

impl Manager {
    pub(crate) fn new(_name: &str) -> Self {
        Self {}
    }

    pub(crate) fn create(&self, _resources: &Resources) -> Result<()> {
        Ok(())
    }

    pub(crate) fn add_task(&self, _tid: i32) -> Result<()> {
        Ok(())
    }
}

impl Drop for Manager {
    fn drop(&mut self) {}
}
