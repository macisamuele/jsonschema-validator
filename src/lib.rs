#![deny(
    unreachable_pub,
    anonymous_parameters,
    bad_style,
    const_err,
    dead_code,
    deprecated,
    illegal_floating_point_literal_pattern,
    improper_ctypes,
    incoherent_fundamental_impls,
    late_bound_lifetime_arguments,
    missing_copy_implementations,
    missing_debug_implementations,
    // missing_docs,
    non_shorthand_field_patterns,
    non_upper_case_globals,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unreachable_code,
    unreachable_patterns,
    unsafe_code,
    unused_allocation,
    unused_assignments,
    unused_comparisons,
    unused_doc_comments,
    unused_extern_crates,
    unused_extern_crates,
    unused_import_braces,
    unused_import_braces,
    unused_imports,
    unused_macros,
    unused_parens,
    unused_qualifications,
    unused_results,
    unused_unsafe,
    unused_variables,
    warnings,
)]
// Feature needed for type_to_str
#![feature(core_intrinsics)]
// Ignore missing_const_for_fn clippy linter (it's too noisy in regards const fn in traits)
#![allow(clippy::missing_const_for_fn)]

#[macro_use]
extern crate strum_macros;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[cfg(all(test, any(feature = "json", feature = "yaml")))]
#[macro_use]
extern crate serde_json;

use std::intrinsics::type_name;

#[macro_use]
mod macros;
#[cfg_attr(test, macro_use)]
pub mod testing_helpers;
#[macro_use]
pub mod types;

pub mod cache;
pub mod drafts;
pub mod keywords;
pub mod loaders;
pub mod prelude;
pub mod schema;
#[cfg(test)]
pub mod testing_prelude;
pub mod url_helpers;

#[inline]
pub(crate) fn type_to_str<T: ?Sized>() -> &'static str {
    // Helper method to get the string name of a given type.
    // NOTE: this currently this is just a proxy to an internal unsafe feature
    //       but having this helper avoids me to eventually spread unsafe on the
    //       codebase more than needed and if there will be a crate that helps us
    //       in achieving the same goal I'll need to modify a single function

    #[allow(unsafe_code)]
    unsafe {
        type_name::<T>()
    }
}
