//! Idmapped bind mounts.
//!
//! Only a process with CAP_SYS_ADMIN in the initial user namespace can set an
//! idmap, and only on a mount that is not attached yet. So the main process,
//! running as root, clones each source tree and sets its idmap before it forks,
//! and the container process moves the detached mount into place.

use nix::sched::CloneFlags;
use nix::unistd::{self, ForkResult, Uid};
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::{self, File};
use std::io::prelude::*;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::{Container, MountOptions, Namespace, error::*};

/// Creates a user namespace with `uidmaps` and `gidmaps`, and returns an fd
/// that refers to it. Each map is `(container_id, host_id, size)`, as in
/// [`Container::uidmaps`].
///
/// The fd is meant for [`Container::idmap`]. For an idmapped mount a map reads
/// from disk to kernel: `(0, K, 1)` makes a file whose on-disk owner is 0
/// appear owned by kernel id K, and a file that kernel id K creates is stored
/// on disk as 0.
///
/// The main process must run as root.
///
/// [`Container::uidmaps`]: crate::Container::uidmaps
/// [`Container::idmap`]: crate::Container::idmap
pub fn idmap_userns(uidmaps: &[(u32, u32, u32)], gidmaps: &[(u32, u32, u32)]) -> Result<OwnedFd> {
    if !Uid::effective().is_root() {
        let err = "idmap_userns: an idmapped mount needs a main process running as root";
        Err(Error::UnError(err.to_string()))?;
    }

    // The helper unshares its user namespace, says so on `ready`, and waits
    // on `done` until the namespace has been opened.
    let (mut ready_r, mut ready_w) = std::io::pipe().map_err(ProcessErrorKind::StdIoError)?;
    let (mut done_r, done_w) = std::io::pipe().map_err(ProcessErrorKind::StdIoError)?;

    match unsafe { unistd::fork() } {
        Ok(ForkResult::Child) => {
            drop(ready_r);
            drop(done_w);
            let status = match nix::sched::unshare(CloneFlags::CLONE_NEWUSER) {
                Ok(_) => 0,
                Err(_) => 1,
            };
            _ = ready_w.write_all(&[status]);
            // Returns at EOF, once the main process has dropped its end.
            let mut buf = [0];
            _ = done_r.read(&mut buf);
            unsafe { libc::_exit(0) }
        }
        Ok(ForkResult::Parent { child }) => {
            drop(ready_w);
            drop(done_r);
            let result = (|| -> Result<OwnedFd> {
                let mut status = [1];
                ready_r
                    .read_exact(&mut status)
                    .map_err(ProcessErrorKind::StdIoError)?;
                if status[0] != 0 {
                    let err = "idmap_userns: unshare(CLONE_NEWUSER) failed";
                    Err(Error::UnError(err.to_string()))?;
                }
                // Root holds CAP_SETUID and CAP_SETGID in the parent namespace,
                // so it writes both maps itself.
                write_idmap(child, "uid_map", uidmaps)?;
                write_idmap(child, "gid_map", gidmaps)?;
                let path = format!("/proc/{child}/ns/user");
                let file = File::open(&path)
                    .map_err(|err| Error::UnError(format!("idmap_userns: open {path}: {err}")))?;
                Ok(OwnedFd::from(file))
            })();
            drop(done_w);
            _ = nix::sys::wait::waitpid(child, None);
            result
        }
        Err(err) => Err(ProcessErrorKind::NixError(err))?,
    }
}

fn write_idmap(child: nix::unistd::Pid, file: &str, idmaps: &[(u32, u32, u32)]) -> Result<()> {
    if idmaps.is_empty() {
        return Ok(());
    }
    let path = format!("/proc/{child}/{file}");
    let lines: String = idmaps
        .iter()
        .map(|(container_id, host_id, size)| format!("{container_id} {host_id} {size}\n"))
        .collect();
    fs::write(&path, lines).map_err(|err| {
        let errmsg = format!("{path}: {err}");
        ProcessErrorKind::SetupUGidmapFailed(errmsg)
    })?;
    Ok(())
}

/// Clones the source of every IDMAP mount as a detached mount and sets its
/// idmap and flags, keyed by container target. The container process moves
/// each into place.
pub(crate) fn mainp_open_trees(container: &Container) -> Result<HashMap<String, OwnedFd>> {
    let mut trees = HashMap::new();
    if !container.namespaces.contains(&Namespace::Mount) {
        return Ok(trees);
    }

    for mount in container.get_mounts() {
        if !mount.options.contains(MountOptions::IDMAP) {
            continue;
        }
        let errmsg =
            |err: &str| format!("idmapped mount {} -> {}: {err}", mount.source, mount.target);
        if !mount.options.contains(MountOptions::BIND) || !mount.fstype.is_empty() {
            Err(Error::UnError(errmsg("IDMAP needs a bind mount")))?;
        }
        let Some(userns) = &container.idmap_userns else {
            return Err(Error::UnError(errmsg(
                "no user namespace, see Container::idmap",
            )));
        };
        if !Uid::effective().is_root() {
            Err(Error::UnError(errmsg(
                "needs a main process running as root",
            )))?;
        }

        let recursive = mount.options.contains(MountOptions::REC);
        let tree = open_tree_clone(Path::new(&mount.source), recursive)
            .map_err(|err| Error::UnError(errmsg(format!("open_tree: {err}").as_str())))?;

        let mut attr_set = libc::MOUNT_ATTR_IDMAP;
        for (option, attr) in [
            (MountOptions::RDONLY, libc::MOUNT_ATTR_RDONLY),
            (MountOptions::NOSUID, libc::MOUNT_ATTR_NOSUID),
            (MountOptions::NODEV, libc::MOUNT_ATTR_NODEV),
            (MountOptions::NOEXEC, libc::MOUNT_ATTR_NOEXEC),
        ] {
            if mount.options.contains(option) {
                attr_set |= attr;
            }
        }
        mount_setattr_idmap(&tree, attr_set, userns, recursive)
            .map_err(|err| Error::UnError(errmsg(format!("mount_setattr: {err}").as_str())))?;
        trees.insert(mount.target.clone(), tree);
    }
    Ok(trees)
}

fn open_tree_clone(source: &Path, recursive: bool) -> std::io::Result<OwnedFd> {
    let source = CString::new(source.as_os_str().as_bytes())?;
    let mut flags = libc::OPEN_TREE_CLONE | libc::OPEN_TREE_CLOEXEC;
    if recursive {
        flags |= libc::AT_RECURSIVE as libc::c_uint;
    }
    let fd = unsafe { libc::syscall(libc::SYS_open_tree, libc::AT_FDCWD, source.as_ptr(), flags) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(unsafe { OwnedFd::from_raw_fd(fd as libc::c_int) })
}

fn mount_setattr_idmap(
    tree: &OwnedFd,
    attr_set: u64,
    userns: &OwnedFd,
    recursive: bool,
) -> std::io::Result<()> {
    let attr = libc::mount_attr {
        attr_set,
        attr_clr: 0,
        // The clone of a shared mount joins its peer group; a private mount
        // keeps the container's mount events out of the host and the host's
        // out of the container, as the plain binds under the private root do.
        propagation: libc::MS_PRIVATE as u64,
        userns_fd: userns.as_raw_fd() as u64,
    };
    let mut flags = libc::AT_EMPTY_PATH as libc::c_uint;
    if recursive {
        flags |= libc::AT_RECURSIVE as libc::c_uint;
    }
    let ret = unsafe {
        libc::syscall(
            libc::SYS_mount_setattr,
            tree.as_raw_fd(),
            c"".as_ptr(),
            flags,
            &attr as *const libc::mount_attr,
            std::mem::size_of::<libc::mount_attr>(),
        )
    };
    if ret < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}
