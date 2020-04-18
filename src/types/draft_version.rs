#[allow(clippy::pub_enum_variant_names)]
#[derive(Clone, Copy, EnumIter, Debug, Display, PartialEq)]
pub(in crate) enum DraftVersion {
    Draft4,
}

impl Default for DraftVersion {
    fn default() -> Self {
        Self::Draft4
    }
}
