maybe_import_dependencies_for_parallel_run!();

#[cfg(any(test, feature = "testing"))]
pub mod testing;

mod builder;
mod scope;
mod validation_errors;

pub use self::builder::Builder as SchemaBuilder;
pub use self::builder::Error as BuilderError;
pub use self::scope::ScopedSchema;
pub use self::validation_errors::ValidationError;
use crate::keywords::KeywordKind;
use crate::keywords::KeywordTrait;
use crate::loaders::Loader;
use crate::loaders::LoaderError;
use crate::types::PrimitiveType;
use std::sync::Arc;
use url::Url;

#[derive(Debug)]
pub struct Schema<T>
where
    T: 'static + PrimitiveType<T>,
{
    keywords: Vec<Arc<KeywordTrait<T>>>,
    path: Url,
    raw_schema: Box<T>,
}

#[allow(unsafe_code)]
unsafe impl<T> Sync for Schema<T> where T: PrimitiveType<T> {}

#[allow(unsafe_code)]
unsafe impl<T> Send for Schema<T> where T: PrimitiveType<T> {}

impl<T> Schema<T>
where
    T: PrimitiveType<T>,
{
    pub fn path(&self) -> &Url {
        &self.path
    }

    #[cfg(feature = "parallel")]
    pub fn keyword(&self, keyword_kind: KeywordKind) -> Option<Arc<KeywordTrait<T>>> {
        self.keywords.par_iter().find_any(|&keyword| keyword_kind == keyword.kind()).cloned()
    }

    #[cfg(not(feature = "parallel"))]
    pub fn keyword(&self, keyword_kind: KeywordKind) -> Option<Arc<KeywordTrait<T>>> {
        self.keywords.iter().find(|&keyword| keyword_kind == keyword.kind()).cloned()
    }

    pub fn new<L>(scoped_schema: &ScopedSchema<T, L>, path: &Url, raw_schema: Box<T>) -> Result<Self, Option<ValidationError<T>>>
    where
        L: Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        match scoped_schema.config().draft_version.get_keywords(scoped_schema, path, &raw_schema) {
            Ok(keywords) => Ok(Self {
                keywords,
                path: path.clone(),
                raw_schema,
            }),
            Err(validation_error) => Err(validation_error),
        }
    }

    pub fn validation_error(&self, value: &T) -> Option<ValidationError<T>> {
        iterate![self.keywords]
            .filter_map(|keyword| keyword.validation_error(value))
            .collect::<Vec<ValidationError<T>>>()
            .pop()
    }
}
