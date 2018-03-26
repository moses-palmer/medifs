use std::io;

use libc;

/// Maps an error kind to an `errno` number.
///
/// # Arguments
/// *  `error` - The error to map.
pub fn map_error(error: io::Error) -> i32 {
    match error.kind() {
        io::ErrorKind::NotFound => libc::ENOENT,
        io::ErrorKind::PermissionDenied => libc::EACCES,
        io::ErrorKind::ConnectionRefused => libc::ECONNREFUSED,
        io::ErrorKind::ConnectionReset => libc::ECONNRESET,
        io::ErrorKind::ConnectionAborted => libc::ECONNABORTED,
        io::ErrorKind::NotConnected => libc::ENOTCONN,
        io::ErrorKind::AddrInUse => libc::EADDRINUSE,
        io::ErrorKind::AddrNotAvailable => libc::EADDRNOTAVAIL,
        io::ErrorKind::BrokenPipe => libc::EPIPE,
        io::ErrorKind::AlreadyExists => libc::EEXIST,
        io::ErrorKind::WouldBlock => libc::EAGAIN,
        io::ErrorKind::InvalidInput => libc::EINVAL,
        io::ErrorKind::InvalidData => libc::EINVAL,
        io::ErrorKind::TimedOut => libc::ETIMEDOUT,
        io::ErrorKind::Interrupted => libc::EINTR,
        _ => libc::EIO,
    }
}
