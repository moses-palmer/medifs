/// Acquires a read lock on a cache.
///
/// If locking fails, this macro will cause the current method to return an
/// error.
macro_rules! cache {
    ($cache:expr) => {
        if let Ok(cache) = $cache.as_ref().cache.read() {
            cache
        } else {
            return Err(libc::EDEADLK);
        }
    }
}


/// Performs a lookup in a cache.
///
/// If no corresponding entry exists, this macro will cause the current method
/// to return an error.
macro_rules! lookup {
    ($cache:expr, $path:expr) => {
        if let Some(entry) = $cache.lookup($path) {
            entry
        } else {
            return Err(libc::ENOENT);
        }
    }
}
