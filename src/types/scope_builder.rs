use crate::types::{draft_version::DraftVersion, schema::Schema, schema_error::SchemaError, scope::Scope};
use json_trait_rs::JsonType;
#[cfg(test)]
use json_trait_rs::RustType;
use loader_rs::{LoaderError, LoaderTrait};
use std::{collections::HashMap, sync::Arc};
use url::Url;
use uuid::Uuid;

#[derive(Debug)]
pub(in crate) struct ScopeBuilder<T: JsonType> {
    pub(in crate) draft_version: DraftVersion,
    loader: Box<dyn LoaderTrait<T>>,
    schema_cache: HashMap<Url, Arc<Schema>>,
}

fn generate_random_url() -> Url {
    Url::parse("memory://").unwrap().join(&Uuid::new_v4().to_string()).unwrap()
}

impl<T: JsonType> ScopeBuilder<T> {
    pub(in crate) fn create<L>(draft_version: DraftVersion, loader: L) -> Self
    where
        L: 'static + LoaderTrait<T>,
    {
        Self {
            draft_version,
            loader: Box::new(loader),
            schema_cache: HashMap::new(),
        }
    }

    pub(in crate) fn retrieve_schema(&mut self, path: &Url) -> Result<Arc<T>, LoaderError> {
        self.loader.get_or_fetch_with_result(path)
    }

    pub(in crate) fn inject_schema(&mut self, raw_schema: &Arc<T>) -> Url {
        let generated_url = generate_random_url();
        self.loader.save_in_cache(&generated_url, raw_schema);
        generated_url
    }

    pub(in crate) fn schema<J: JsonType>(&mut self, path: &Url, raw_schema: &J) -> Result<Arc<Schema>, SchemaError>
    where
        T: 'static,
    {
        if let Some(cached_schema) = self.schema_cache.get(path) {
            Ok(cached_schema.clone())
        } else {
            let arc_schema: Arc<Schema> = Arc::new(Schema::create(self, path, raw_schema)?);
            let _ = self.schema_cache.insert(path.clone(), arc_schema.clone());
            Ok(arc_schema)
        }
    }

    pub(in crate) fn build(&mut self) -> Scope {
        self.schema_cache.values_mut().for_each(|schema_arc| {
            #[allow(unsafe_code)]
            unsafe { Arc::get_mut_unchecked(schema_arc) }.initialise();
        });

        Scope {
            draft_version: self.draft_version,
            schema_cache: self.schema_cache.clone(),
        }
    }
}

#[cfg(test)]
pub(in crate::types) fn scope_builder_create<A>(
    draft_version: DraftVersion,
    raw_schema: RustType,
    closure: &dyn Fn(&mut ScopeBuilder<RustType>, &Url, &RustType) -> A,
) -> (ScopeBuilder<RustType>, A) {
    let loader = loader_rs::loaders::RustTypeLoader::default();
    let mut scope_builder: ScopeBuilder<RustType> = ScopeBuilder::create(draft_version, loader);
    let arc_raw_schema = Arc::new(raw_schema);
    let generated_url = scope_builder.inject_schema(&arc_raw_schema);
    let closure_result = closure(&mut scope_builder, &generated_url, &arc_raw_schema);
    (scope_builder, closure_result)
}

#[cfg(test)]
pub(in crate::types) fn scope_builder_create_and_build<A>(
    draft_version: DraftVersion,
    raw_schema: RustType,
    closure: &dyn Fn(&mut ScopeBuilder<RustType>, &Url, &RustType) -> A,
) -> A {
    let (mut scope_builder, closure_result) = scope_builder_create(draft_version, raw_schema, closure);
    let _ = scope_builder.build();
    closure_result
}
