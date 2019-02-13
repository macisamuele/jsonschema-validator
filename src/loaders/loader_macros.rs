#[macro_export]
macro_rules! _get_memory_address {
    ($reference:expr) => {
        ($reference as *const _) as isize
    };
}

#[macro_export]
macro_rules! setup_loader {
    ($import_type:path, $primitive_type:ty, $loader_name:ident, $format_error:ty, $load_from_string_callable:expr,) => {
        use crate::cache::Cache;
        use crate::loaders::Loader;
        use crate::loaders::LoaderError;
        use url;
        use $import_type;

        #[derive(Clone, Debug, PartialEq)]
        pub struct $loader_name {
            cache: Cache<url::Url, $primitive_type>,
        }

        impl Default for $loader_name {
            fn default() -> Self {
                Self { cache: Cache::default() }
            }
        }

        impl Loader<$primitive_type> for $loader_name {
            type FormatError = $format_error;

            fn cache(&self) -> &Cache<url::Url, $primitive_type> {
                &self.cache
            }

            #[inline]
            fn load_from_string(content: String) -> Result<$primitive_type, LoaderError<Self::FormatError>> {
                $load_from_string_callable(content)
            }
        }

        impl From<$format_error> for LoaderError<$format_error> {
            fn from(error: $format_error) -> Self {
                LoaderError::FormatError(error)
            }
        }
    };
}
