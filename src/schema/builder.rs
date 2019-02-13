use crate::cache::Cached;
use crate::drafts::DraftVersion;
use crate::loaders::Loader;
use crate::loaders::LoaderError;
use crate::schema::ScopedSchema;
use crate::schema::ValidationError;
use crate::type_to_str;
use crate::types::PrimitiveType;
use crate::url_helpers::normalize_url_for_cache;
use crate::url_helpers::parse_and_normalize_url;
use url::Url;

#[derive(Debug)]
pub struct Builder<T, L>
where
    T: 'static + PrimitiveType<T>,
    L: 'static + Loader<T>,
    LoaderError<L::FormatError>: From<L::FormatError>,
{
    base_uri: Option<Url>,
    draft_version: Option<DraftVersion>,
    follow_references: bool,
    raw_schema: Option<Box<T>>,
    loader: L,
    validate_schema: bool,
}

#[derive(Debug)]
pub enum Error<T, L>
where
    T: PrimitiveType<T>,
    L: Loader<T>,
    LoaderError<L::FormatError>: From<L::FormatError>,
{
    Builder(String),
    Schema(Option<ValidationError<T>>),
    Loader(L::FormatError),
}

impl<T, L> Builder<T, L>
where
    T: 'static + PrimitiveType<T>,
    L: 'static + Loader<T>,
    LoaderError<L::FormatError>: From<L::FormatError>,
{
    // TODO: add support for custom format validators (other than the defaults)

    pub fn draft_version(&mut self, draft_version: DraftVersion) -> &mut Self {
        self.draft_version = Some(draft_version);
        self
    }

    pub fn follow_references(&mut self, follow_references: bool) -> &mut Self {
        self.follow_references = follow_references;
        self
    }

    pub fn raw_schema(&mut self, raw_schema: Box<T>) -> &mut Self {
        let base_uri = normalize_url_for_cache(&parse_and_normalize_url(&format!("memory://{:p}", raw_schema)).unwrap());
        self.base_uri = Some(base_uri.clone());

        {
            // Internal context needed to ensure mutable borrow drop
            let cache = self.loader.cache();
            cache.clear(); // Ensure that loader cache is clean
            cache.set(base_uri, *raw_schema.clone());
        }

        self.raw_schema = Some(raw_schema);
        self
    }

    pub fn raw_schema_from_uri<R: AsRef<str>>(&mut self, raw_schema_uri: R) -> Result<&mut Self, LoaderError<L::FormatError>> {
        self.loader.cache().clear(); // Ensure that loader cache is clean
        self.loader.load(raw_schema_uri.as_ref()).and_then(|raw_schema| {
            self.base_uri = Some(parse_and_normalize_url(raw_schema_uri).unwrap());
            self.raw_schema = Some(Box::new(raw_schema));
            Ok(self)
        })
    }

    pub fn validate_schema(&mut self, validate_schema: bool) -> &mut Self {
        self.validate_schema = validate_schema;
        self
    }

    pub fn build(&mut self) -> Result<ScopedSchema<T, L>, Error<T, L>> {
        let draft_version = if let Some(value) = self.draft_version {
            value
        } else {
            return Err(Error::Builder(format!(
                "Builder requires draft version to be defined, please use {}::draft_version",
                type_to_str::<Self>(),
            )));
        };

        let raw_schema = if let Some(value) = &self.raw_schema {
            value
        } else {
            return Err(Error::Builder(format!(
                "Builder requires raw schema to be defined, please use {}::raw_schema or {}::raw_schema_from_uri`",
                type_to_str::<Self>(),
                type_to_str::<Self>(),
            )));
        };

        let base_uri = if let Some(value) = &self.base_uri {
            value.clone()
        } else {
            return Err(Error::Builder(
                "No url defined while building the Schema object. This should not happen. Please make sure to open an issue on the GitHub project setting all the steps to reproduce it.".to_string()
            ));
        };

        match ScopedSchema::new(
            draft_version,
            self.follow_references,
            self.loader.clone(),
            self.validate_schema,
            raw_schema.clone(),
            &base_uri,
        ) {
            Ok(scoped_schema) => Ok(scoped_schema),
            Err(validation_error) => Err(Error::Schema(validation_error)),
        }
    }
}

impl<T, L> Default for Builder<T, L>
where
    T: 'static + PrimitiveType<T>,
    L: 'static + Loader<T>,
    LoaderError<L::FormatError>: From<L::FormatError>,
{
    #[inline]
    fn default() -> Self {
        Self {
            base_uri: None,
            draft_version: None,
            follow_references: false,
            loader: L::default(),
            raw_schema: None,
            validate_schema: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Builder;
    use super::Error;
    use crate::testing_prelude::*;
    use crate::type_to_str;

    #[test]
    fn test_builder_fail_to_build_when_missing_draft_version() {
        expected_err!(
            Builder::<TestingType, TestingLoader>::default().raw_schema(Box::new(TestingType::default())).build(),
            Error::Builder,
            |value| {
                assert_eq!(
                    value,
                    format!(
                        "Builder requires draft version to be defined, please use {}::draft_version",
                        type_to_str::<Builder<TestingType, TestingLoader>>(),
                    ),
                );
            }
        );
    }

    #[test]
    fn test_builder_fails_if_schema_is_not_valid() {
        expected_err!(
            Builder::<_, TestingLoader>::default()
                .draft_version(DraftVersion::Draft4)
                .follow_references(false)
                .validate_schema(true)
                .raw_schema(Box::new(testing_map!["type" => 1]))
                .build(),
            Error::Schema,
            |optional_validation_error: Option<ValidationError<TestingType>>| {
                assert_eq!(optional_validation_error.and_then(|item| Some(item.keyword_kind)), Some(KeywordKind::Type));
            }
        );
    }

    #[test]
    fn test_builder_builds_when_schema_is_defined() {
        let _ = Builder::<_, TestingLoader>::default()
            .draft_version(DraftVersion::Draft4)
            .follow_references(false)
            .validate_schema(true)
            .raw_schema(Box::new(TestingType::default()))
            .build()
            .expect("No errors are expected");
    }
}
