use close_fds::close_open_fds;
use std::io::{PipeReader, PipeWriter};
use std::os::fd::AsRawFd;

use super::error::*;

pub(crate) fn close_extra_fds_exclude(reader: &PipeReader, writer: &PipeWriter) -> Result<()> {
    let mut keep_fds = [reader.as_raw_fd(), writer.as_raw_fd()];
    keep_fds.sort_unstable();

    unsafe {
        close_open_fds(3, &keep_fds);
    }
    Ok(())
}
