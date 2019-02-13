maybe_import_dependencies_for_parallel_run!();

use crate::drafts::DraftVersion;
use crate::loaders::Loader;
use crate::loaders::LoaderError;
use crate::schema::builder::Error;
use crate::schema::SchemaBuilder;
use crate::schema::ScopedSchema;
use crate::schema::ValidationError;
use crate::types::testing::TestingType;
use crate::types::PrimitiveType;
use crate::url_helpers::parse_and_normalize_url;
use url::Url;

pub fn normalize_urls_in_validation_errors<T, R>(optional_validation_error: Option<ValidationError<T>>, new_host: R) -> Option<ValidationError<T>>
where
    T: PrimitiveType<T>,
    R: AsRef<str>,
{
    optional_validation_error.and_then(|mut error| {
        error.path.set_host(Some(new_host.as_ref())).unwrap();
        Some(error)
    })
}

pub fn create_scoped_schema_from_raw_schema<T, L>(raw_schema: &T, validate_schema: bool) -> Result<ScopedSchema<T, L>, Option<ValidationError<T>>>
where
    T: PrimitiveType<T>,
    L: Loader<T>,
    LoaderError<L::FormatError>: From<L::FormatError>,
{
    match SchemaBuilder::default()
        .draft_version(DraftVersion::Draft4)
        .follow_references(true)
        .raw_schema(Box::new(raw_schema.clone()))
        .validate_schema(validate_schema)
        .build()
    {
        Err(Error::Schema(validation_errors)) => Err(validation_errors),
        Ok(scoped_schema) => Ok(scoped_schema),
        Err(unexpected_error) => panic!("Builder error: {:?}", unexpected_error),
    }
}

pub fn get_path_from_scoped_schema<L>(scoped_schema: &ScopedSchema<TestingType, L>) -> Url
where
    L: Loader<TestingType>,
    LoaderError<L::FormatError>: From<L::FormatError>,
{
    match scoped_schema.root_schema {
        Some(ref schema) => schema.path().clone(),
        None => parse_and_normalize_url("memory://HOST").unwrap(),
    }
}
