use crate::types::{draft_version::DraftVersion, schema::Schema};
use std::{collections::HashMap, sync::Arc};
use url::Url;

pub(in crate) struct Scope {
    pub(in crate) draft_version: DraftVersion,
    // TODO: Verify if we need a thread-safe cache or this is good enough
    pub(in crate) schema_cache: HashMap<Url, Arc<Schema>>,
}
