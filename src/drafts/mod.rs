use crate::keywords::KeywordTrait;
use crate::loaders::Loader;
use crate::loaders::LoaderError;
use crate::schema::ScopedSchema;
use crate::schema::ValidationError;
use crate::types::PrimitiveType;
use std::sync::Arc;
use url::Url;

#[macro_use]
mod macros;

mod draft4;

#[allow(clippy::pub_enum_variant_names)]
#[derive(Clone, Copy, EnumIter, Debug, Display, PartialEq)]
pub enum DraftVersion {
    Draft3,
    Draft4,
    Draft6,
    Draft7,
}

impl DraftVersion {
    #[allow(clippy::type_complexity)]
    pub fn get_keywords<T, L>(self, scoped_schema: &mut ScopedSchema<T, L>, path: &Url, raw_schema: &T) -> Result<Vec<Arc<KeywordTrait<T>>>, Option<ValidationError<T>>>
    where
        T: 'static + PrimitiveType<T>,
        L: 'static + Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        match self {
            DraftVersion::Draft4 => draft4::Keywords::create(path, scoped_schema, raw_schema),
            _ => panic!("{} is not fully supported yet", self),
        }
    }

    pub fn schema_url(draft: Self) -> String {
        format!(
            "http://json-schema.org/draft-{:02}/schema#",
            match draft {
                DraftVersion::Draft4 => 4,
                _ => panic!("{} is not fully supported yet", draft),
            }
        )
    }
}
