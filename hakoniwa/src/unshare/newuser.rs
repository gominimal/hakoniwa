mod idmap;

use nix::unistd::Pid;
use std::process::Command;

use crate::{Container, error::*};

pub(crate) use idmap::IdMap;

pub(crate) fn mainp_setup(container: &Container, child: Pid) -> Result<()> {
    // A main process running as root holds CAP_SETUID and CAP_SETGID in the
    // parent namespace, so the kernel lets it write any map itself, and no
    // setuid helper has to be installed for it.
    if nix::unistd::Uid::effective().is_root() {
        let uidmaps = container.uidmaps.clone().unwrap_or_default();
        mainp_write_idmap("uid_map", &uidmaps, child)?;
        let gidmaps = container.gidmaps.clone().unwrap_or_default();
        mainp_write_idmap("gid_map", &gidmaps, child)?;
        log::debug!("================================");
        return Ok(());
    }

    let program = "newuidmap";
    let uidmaps = container.uidmaps.clone().unwrap_or_default();
    mainp_setup_newidmap(program, &uidmaps, child)?;

    let program = "newgidmap";
    let gidmaps = container.gidmaps.clone().unwrap_or_default();
    mainp_setup_newidmap(program, &gidmaps, child)?;

    log::debug!("================================");
    Ok(())
}

fn mainp_write_idmap(file: &str, idmaps: &[IdMap], child: Pid) -> Result<()> {
    if idmaps.is_empty() {
        return Ok(());
    }

    let path = format!("/proc/{child}/{file}");
    let lines: String = idmaps.iter().map(IdMap::to_line).collect();
    log::debug!("Configuring UID/GID mapping: Write: {path}: {lines:?}");
    std::fs::write(&path, lines).map_err(|err| {
        let errmsg = format!("{path}: {err}");
        ProcessErrorKind::SetupUGidmapFailed(errmsg)
    })?;
    Ok(())
}

fn mainp_setup_newidmap(program: &str, idmaps: &[IdMap], child: Pid) -> Result<()> {
    if idmaps.is_empty() {
        return Ok(());
    }

    let cmdline = newidmap_cmdline(program, idmaps, child);
    log::debug!("Configuring UID/GID mapping: Execve: {cmdline:?}");

    let output = Command::new(cmdline[0].clone())
        .args(&cmdline[1..])
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let output = String::from_utf8_lossy(&output.stdout).trim().to_string();
            log::debug!("Configuring UID/GID mapping: Output: {output}");
            Ok(())
        }
        Ok(output) => {
            let errmsg = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(ProcessErrorKind::SetupUGidmapFailed(errmsg))?
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let errmsg = format!("command {program:?} not found");
            Err(ProcessErrorKind::SetupUGidmapFailed(errmsg))?
        }
        Err(err) => {
            let errmsg = format!("{err}");
            Err(ProcessErrorKind::SetupUGidmapFailed(errmsg))?
        }
    }
}

fn newidmap_cmdline(program: &str, idmaps: &[IdMap], child: Pid) -> Vec<String> {
    let mut idmaps = idmaps
        .iter()
        .flat_map(|idmap| {
            [
                idmap.container_id.to_string(),
                idmap.host_id.to_string(),
                idmap.size.to_string(),
            ]
        })
        .collect();

    let mut cmdline = vec![program.to_string(), child.to_string()];
    cmdline.append(&mut idmaps);
    cmdline
}
