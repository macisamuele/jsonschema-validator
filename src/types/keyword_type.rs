#[derive(Clone, Copy, Eq, Debug, Display, PartialEq)]
pub(in crate) enum KeywordType {
    Unknown,
    Type,
    Properties,
    Ref,
    Required,
}
