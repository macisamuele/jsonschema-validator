maybe_import_dependencies_for_parallel_run!();

use super::Schema;
use crate::cache::Cache;
use crate::cache::Cached;
use crate::drafts::DraftVersion;
use crate::keywords::KeywordKind;
use crate::keywords::KeywordTrait;
use crate::loaders::Loader;
use crate::loaders::LoaderError;
use crate::schema::ValidationError;
use crate::types::PrimitiveType;
use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub draft_version: DraftVersion,
    pub follow_references: bool,
    pub validate_schema: bool,
}

#[derive(Debug)]
pub struct ScopedSchema<T, L>
where
    T: 'static + PrimitiveType<T>,
    L: 'static + Loader<T>,
    LoaderError<L::FormatError>: From<L::FormatError>,
{
    pub(crate) config: Config,
    pub(crate) loader: L,
    pub(crate) root_schema: Option<Schema<T>>,
    pub(crate) schema_cache: Cache<Url, Schema<T>>,
}

#[allow(unsafe_code)]
unsafe impl<T, L> Sync for ScopedSchema<T, L>
where
    T: PrimitiveType<T>,
    L: Loader<T>,
    LoaderError<L::FormatError>: From<L::FormatError>,
{
}

#[allow(unsafe_code)]
unsafe impl<T, L> Send for ScopedSchema<T, L>
where
    T: PrimitiveType<T>,
    L: Loader<T>,
    LoaderError<L::FormatError>: From<L::FormatError>,
{
}

impl<T, L> ScopedSchema<T, L>
where
    T: 'static + PrimitiveType<T>,
    L: 'static + Loader<T>,
    LoaderError<L::FormatError>: From<L::FormatError>,
{
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn keyword(&self, keyword_kind: KeywordKind) -> Option<Arc<KeywordTrait<T>>> {
        match &self.root_schema {
            Some(schema) => schema.keyword(keyword_kind),
            None => {
                unreachable!("This section should be not reachable as Self::new initialises it");
            }
        }
    }

    pub fn new(
        draft_version: DraftVersion,
        follow_references: bool,
        loader: L,
        validate_schema: bool,
        raw_schema: Box<T>,
        schema_path: &Url,
    ) -> Result<Self, Option<ValidationError<T>>> {
        let mut scoped_schema = Self {
            config: Config {
                draft_version,
                follow_references,
                validate_schema,
            },
            loader,
            root_schema: None,
            schema_cache: Cache::default(),
        };
        scoped_schema.root_schema = Some(Schema::new(&scoped_schema, schema_path, raw_schema)?);
        Ok(scoped_schema)
    }

    pub fn get_schema(&self, schema_path: &Url, raw_schema: Box<T>) -> Result<Arc<Schema<T>>, Option<ValidationError<T>>> {
        match self.schema_cache.get(schema_path) {
            Some(cached_schema) => Ok(cached_schema),
            None => {
                let arc_schema = Arc::new(Schema::new(self, schema_path, raw_schema.clone())?);
                self.schema_cache.set_from_arc(schema_path.clone(), arc_schema.clone());
                Ok(arc_schema)
            }
        }
    }

    pub fn validation_error(&self, value: &T) -> Option<ValidationError<T>> {
        match &self.root_schema {
            None => {
                eprintln!("Self::new takes case to initialize it to Some");
                None
            }
            Some(schema) => schema.validation_error(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing_prelude::*;

    lazy_static! {
        static ref INVALID_DRAFT4_SCHEMA: TestingType = testing_map!["type" => 1,];
    }

    #[test]
    fn test_new_schema_should_panic_with_invalid_schema_if_no_validation() {
        let raw_schema = &INVALID_DRAFT4_SCHEMA.clone();

        should_panic!({
            let _: Result<ScopedSchema<TestingType, TestingLoader>, Option<ValidationError<TestingType>>> = create_scoped_schema_from_raw_schema(raw_schema, false);
        });
    }

    #[test]
    fn test_new_schema_fails_with_invalid_schema_if_validation_is_set() {
        let raw_schema = &INVALID_DRAFT4_SCHEMA.clone();
        let scoped_schema: Result<ScopedSchema<TestingType, TestingLoader>, Option<ValidationError<TestingType>>> = create_scoped_schema_from_raw_schema(raw_schema, true);
        assert_eq!(scoped_schema.is_err(), true);
    }
}
