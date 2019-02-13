use crate::keywords::KeywordKind;
use crate::types::PrimitiveType;
use std::marker::PhantomData;
use url::Url;

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationError<T>
where
    T: PrimitiveType<T>,
{
    // TODO: enhance information
    pub keyword_kind: KeywordKind,
    pub path: Url,
    phantom_data: PhantomData<T>,
}

impl<T> Default for ValidationError<T>
where
    T: PrimitiveType<T>,
{
    fn default() -> Self {
        Self {
            keyword_kind: KeywordKind::default(),
            path: Url::parse("memory:///").unwrap(),
            phantom_data: PhantomData,
        }
    }
}

impl<T> ValidationError<T>
where
    T: PrimitiveType<T>,
{
    #[inline]
    pub fn new(keyword_kind: KeywordKind, path: &Url) -> Self {
        Self {
            keyword_kind,
            path: path.clone(),
            phantom_data: PhantomData,
        }
    }
}
