// This module will contain a all the jsonschema keywords
// NOTE: to reduce code duplication all the keywords will be defined on a single namespace
// and then collected into "container" draft specific. A lot of keywords have the same definition
// on multiple drafts

pub mod common;
mod keyword;

pub use self::keyword::Attribute as KeywordAttribute;
pub use self::keyword::Trait as KeywordTrait;

#[derive(Clone, Copy, EnumIter, Debug, Display, PartialEq)]
pub enum KeywordKind {
    Unknown, // Enum value used only for providing a default value
    // keywords will start to underscore to workaround collision between jsonschema and rust keywords
    Type,
    Properties,
}

impl Default for KeywordKind {
    fn default() -> Self {
        KeywordKind::Unknown
    }
}
