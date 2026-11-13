#[cfg(test)]
mod idmap_test {
    use assertables::*;
    use nix::unistd::Uid;
    use std::fs::{self, Permissions};
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::sync::Arc;
    use tempfile::TempDir;

    use hakoniwa::{Container, idmap_userns};

    // The container's own maps: the namespace's root stays root, and uid and
    // gid 1000 map onto kernel id 100000.
    const PRIVILEGED_MAP: [(u32, u32, u32); 2] = [(0, 0, 1), (1000, 100000, 1)];

    // The idmap of the mounts: on-disk id 0 appears as kernel id 100000, so
    // as the container's user 1000.
    const DISK_ROOT_ONTO_MAPPED: [(u32, u32, u32); 1] = [(0, 100000, 1)];

    fn privileged_container() -> Container {
        let mut container = Container::new();
        container.uidmaps(&PRIVILEGED_MAP);
        container.gidmaps(&PRIVILEGED_MAP);
        container
    }

    fn idmap() -> Arc<std::os::fd::OwnedFd> {
        Arc::new(idmap_userns(&DISK_ROOT_ONTO_MAPPED, &DISK_ROOT_ONTO_MAPPED).unwrap())
    }

    // The arguments that run `script` as uid and gid 1000 with no
    // supplementary groups, the way a container user drops from the
    // namespace's root.
    fn as_mapped_user(script: &str) -> [&str; 6] {
        [
            "--reuid=1000",
            "--regid=1000",
            "--clear-groups",
            "/bin/sh",
            "-c",
            script,
        ]
    }

    // A directory that kernel id 0 owns, mode 0755, holding a file `f`
    // (0644) and a directory `d` (0755), all owned by 0:0.
    fn root_owned_dir() -> TempDir {
        let dir = TempDir::with_prefix("hakoniwa-idmap-").unwrap();
        fs::set_permissions(dir.path(), Permissions::from_mode(0o755)).unwrap();
        let file = dir.path().join("f");
        fs::write(&file, "x").unwrap();
        fs::set_permissions(&file, Permissions::from_mode(0o644)).unwrap();
        let subdir = dir.path().join("d");
        fs::create_dir(&subdir).unwrap();
        fs::set_permissions(&subdir, Permissions::from_mode(0o755)).unwrap();
        for path in [dir.path(), file.as_path(), subdir.as_path()] {
            let meta = fs::metadata(path).unwrap();
            assert_eq!((meta.uid(), meta.gid()), (0, 0), "{path:?}");
        }
        dir
    }

    const SCRIPT: &str = "stat -c %u:%g /mnt/f /mnt/d; echo y > /mnt/new";

    #[test]
    fn test_privileged_idmap_userns_has_the_maps() {
        if !Uid::effective().is_root() {
            return;
        }
        let userns = idmap_userns(&DISK_ROOT_ONTO_MAPPED, &DISK_ROOT_ONTO_MAPPED).unwrap();
        let path = format!("/proc/{}/fd/{}", std::process::id(), userns.as_raw_fd());
        let output = std::process::Command::new("nsenter")
            .arg(format!("--user={path}"))
            .arg("--preserve-credentials")
            .args(["cat", "/proc/self/uid_map", "/proc/self/gid_map"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<Vec<&str>> = stdout
            .lines()
            .map(|line| line.split_whitespace().collect())
            .collect();
        let map = vec!["0", "100000", "1"];
        assert_eq!(lines, [map.clone(), map], "{stdout}");
    }

    #[test]
    fn test_privileged_idmap_bind_shows_disk_root_as_the_mapped_user() {
        // On-disk 0 appears as the container's user 1000, and a file it
        // creates lands on disk as 0:0.
        if !Uid::effective().is_root() {
            return;
        }
        let dir = root_owned_dir();
        let mut container = privileged_container();
        container.rootfs("/").unwrap();
        container
            .idmap(idmap())
            .bindmount_rw_idmapped(dir.path().to_str().unwrap(), "/mnt");
        let output = container
            .command("/usr/bin/setpriv")
            .args(as_mapped_user(SCRIPT))
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, ["1000:1000", "1000:1000"], "{output:?}");
        let meta = fs::metadata(dir.path().join("new")).unwrap();
        assert_eq!((meta.uid(), meta.gid()), (0, 0));
        assert_eq!(fs::read_to_string(dir.path().join("new")).unwrap(), "y\n");
    }

    #[test]
    fn test_privileged_idmap_plain_bind_shows_disk_root_as_root() {
        // Without the idmap the same files are root's, and the mapped user
        // cannot write into the directory.
        if !Uid::effective().is_root() {
            return;
        }
        let dir = root_owned_dir();
        let mut container = privileged_container();
        container.rootfs("/").unwrap();
        container.bindmount_rw(dir.path().to_str().unwrap(), "/mnt");
        let output = container
            .command("/usr/bin/setpriv")
            .args(as_mapped_user(SCRIPT))
            .output()
            .unwrap();
        assert!(!output.status.success(), "{output:?}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, ["0:0", "0:0"], "{output:?}");
        assert_contains!(String::from_utf8_lossy(&output.stderr), "Permission denied");
        assert!(!dir.path().join("new").exists());
    }

    #[test]
    fn test_privileged_idmap_rootfs_runs_as_the_mapped_user() {
        // Every directory of the root filesystem is bound through the idmap:
        // `/usr` appears as the mapped user's, and its programs still run.
        if !Uid::effective().is_root() {
            return;
        }
        let mut container = privileged_container();
        container.idmap(idmap()).rootfs_idmapped("/").unwrap();
        let output = container
            .command("/usr/bin/setpriv")
            .args(as_mapped_user("stat -c %u /usr && /bin/true"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "1000");
    }
}
