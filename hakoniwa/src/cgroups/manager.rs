use libcgroups::common::{
    CgroupConfig, CgroupManager, ControllerOpt as CgroupControllerOpt, create_cgroup_manager,
};
use nix::unistd::Pid;
use std::path::PathBuf;

use super::{Resources, error::*};

#[derive(Clone, Debug)]
pub(crate) struct Manager {
    name: String,
}

impl Manager {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub(crate) fn apply(&self, id: i32, resources: &Resources) -> Result<()> {
        let manager = create_cgroup_manager(CgroupConfig {
            cgroup_path: PathBuf::from(format!("user.slice:hakoniwa:{}", self.name)),
            systemd_cgroup: true,
            container_name: self.name.clone(),
        })?;

        let id = Pid::from_raw(id);
        manager.add_task(id)?;

        let opts = CgroupControllerOpt {
            resources: &resources.build()?,
            disable_oom_killer: false,
            oom_score_adj: None,
            freezer_state: None,
        };
        manager.apply(&opts)?;

        Ok(())
    }
}

impl Drop for Manager {
    fn drop(&mut self) {}
}
