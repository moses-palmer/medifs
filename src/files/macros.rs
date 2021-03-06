/// Acquires a read lock on a cache.
///
/// If locking fails, this macro will cause the current method to return
/// [`libc::EDEADLK`].
///
/// [`libc::EDEADLK`]: https://doc.rust-lang.org/libc/x86_64-unknown-linux-gnu/libc/constant.EDEADLK.html
macro_rules! cache {
    ($cache:expr) => {
        if let Ok(cache) = $cache.read() {
            cache
        } else {
            return Err(libc::EDEADLK);
        }
    }
}

/// Performs a lookup in a cache.
///
/// If no corresponding entry exists, this macro will cause the current method
/// to return [`libc::ENOENT`].
///
/// [`libc::ENOENT`]: https://doc.rust-lang.org/libc/x86_64-unknown-linux-gnu/libc/constant.ENOENT.html
macro_rules! lookup {
    ($cache:expr, $path:expr) => {
        if let Some(entry) = $cache.lookup($path) {
            entry
        } else {
            return Err(libc::ENOENT);
        }
    }
}

/// Acquires a write lock on a source and sends a notification.
///
/// If locking fails, this macro will cause the current method to return
/// [`libc::EDEADLK`].
///
/// [`libc::EDEADLK`]: https://doc.rust-lang.org/libc/x86_64-unknown-linux-gnu/libc/constant.EDEADLK.html
macro_rules! notify {
    ($source:expr) => {
        if let Ok(ref mut source) = $source.write() {
            source.notify();
        } else {
            return Err(libc::EDEADLK);
        }
    }
}
