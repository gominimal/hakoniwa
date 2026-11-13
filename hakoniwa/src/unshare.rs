mod idmap;
mod namespace;
mod newnet;
mod newns;
mod newuser;

pub(crate) use newnet::SetupStatus as SetupNetworkStatus;
pub(crate) use newns::{FsMakeDir, FsMakeSymlink, FsOperation, FsWriteFile, Mount};
pub(crate) use newuser::IdMap;

pub use idmap::idmap_userns;
pub use namespace::Namespace;
pub use newnet::{Network, Pasta};
pub use newns::MountOptions;

#[cfg(feature = "rustslirp")]
pub use newnet::rustslirp::RustSlirp;

#[cfg(feature = "rustslirp")]
pub use newnet::rustslirp::RustSlirpMode;

#[cfg(feature = "rustslirp")]
pub use newnet::rustslirp::RustSlirpGateway;

pub(crate) fn mainp_setup_network(
    container: &crate::Container,
    child: nix::unistd::Pid,
) -> crate::error::Result<SetupNetworkStatus> {
    newnet::mainp_setup(container, child)
}

pub(crate) fn mainp_open_idmapped_trees(
    container: &crate::Container,
) -> crate::error::Result<std::collections::HashMap<String, std::os::fd::OwnedFd>> {
    idmap::mainp_open_trees(container)
}

pub(crate) fn mainp_setup_ugidmap(
    container: &crate::Container,
    child: nix::unistd::Pid,
) -> crate::error::Result<()> {
    newuser::mainp_setup(container, child)
}
