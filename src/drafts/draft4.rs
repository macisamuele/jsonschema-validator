// This module will contain a all the jsonschema draft4 keywords
// NOTE: to reduce code duplication all the keywords will be defined on a single namespace
// and then collected into "container" draft specific. A lot of keywords have the same definition
// on multiple drafts
maybe_import_dependencies_for_parallel_run!();

use crate::keywords::common::_properties::Properties;
use crate::keywords::common::_type::Type;

use crate::keywords::KeywordTrait;
use crate::loaders::Loader;
use crate::loaders::LoaderError;
use crate::schema::ScopedSchema;
use crate::schema::ValidationError;
use crate::types::PrimitiveType;
use std::sync::Arc;
use url::Url;

#[derive(Clone, Copy, Debug, Display, EnumIter, PartialEq)]
pub(super) enum Keywords {
    Properties,
    Type,
}

impl Keywords {
    #[allow(clippy::type_complexity)]
    pub(super) fn create<T, L>(path: &Url, scoped_schema: &ScopedSchema<T, L>, raw_schema: &T) -> Result<Vec<Arc<KeywordTrait<T>>>, Option<ValidationError<T>>>
    where
        T: 'static + PrimitiveType<T>,
        L: 'static + Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        let (keywords, mut validation_errors) = initialize_keywords! {
            primitive_type: T,
            scoped_schema: scoped_schema,
            path: path,
            raw_schema: raw_schema,
            keywords: [
                Type,
                Properties,
            ],
        };

        if validation_errors.is_empty() {
            Ok(keywords)
        } else {
            Err(validation_errors.pop())
        }
    }
}
