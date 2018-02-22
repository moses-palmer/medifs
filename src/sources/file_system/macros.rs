/// Implements the basic file system source trait by simply reading attributes
/// from self.
///
/// This macro will also define the concrete struct.
macro_rules! file_system_base {
    ($source_type:ident, $($field_name:ident : $field_type:ty,)*) => {
        pub struct $source_type {
            root: path::PathBuf,
            cache: files::Cache,
            timestamp: Option<std::time::SystemTime>,
            $($field_name: $field_type,)*
        }

        impl FileSystemSource for $source_type {
            fn cache(&self) -> &files::Cache {
                &self.cache
            }

            fn timestamp(&mut self) -> &mut Option<std::time::SystemTime> {
                &mut self.timestamp
            }

            fn root(&self) -> &path::PathBuf {
                &self.root
            }
        }
    }
}
