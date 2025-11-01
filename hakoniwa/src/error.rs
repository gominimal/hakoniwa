/// Error handling with the Result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ProcessError(#[from] ProcessErrorKind),

    #[error("{0}")]
    UnError(String),

    #[error(transparent)]
    EtcfsError(#[from] EtcfsErrorKind),
}

#[derive(thiserror::Error, Debug)]
pub enum ProcessErrorKind {
    #[error(transparent)]
    BincodeDecodeError(#[from] bincode::error::DecodeError),
    #[error(transparent)]
    NixError(#[from] nix::Error),
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
    #[error("thread panic")]
    StdThreadPanic,
    #[error("configure the UID/GID mapping of a user namespace failed: {0}")]
    SetupUGidmapFailed(String),
    #[error("configure the new network namespace failed: {0}")]
    SetupNetworkFailed(String),
    #[error("child exit status gone")]
    ChildExitStatusGone,
}

#[derive(thiserror::Error, Debug)]
pub enum EtcfsErrorKind {
    #[error("parse line `{line}..` failed: {errmsg}")]
    InvalidLine { line: String, errmsg: String },
    #[error("not enough parts")]
    NotEnoughParts,
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
    #[error(transparent)]
    StdNumParseIntError(#[from] std::num::ParseIntError),
}
