#![deny(
    unreachable_pub,
    anonymous_parameters,
    bad_style,
    const_err,
    dead_code,
    deprecated,
    illegal_floating_point_literal_pattern,
    improper_ctypes,
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
// Enable very pendantic clippy linting
#![deny(clippy::pedantic, clippy::nursery)]
// Enable generator features (crate::iterator_utils::generator_iterator)
#![feature(generators, generator_trait)]
// Enable Arc::get_mut_unchecked for crate::types::scope_builder::ScopeBuilder::build
#![feature(get_mut_unchecked)]
// Enable str::strip_suffix for crate::types::validation_error::normalise_path
#![feature(str_strip)]
// Enable is_empty() on ExactSizeIterator instances
#![feature(exact_size_is_empty)]
#![allow(dead_code)] // TODO: Remove this. This is a temporary patch to allow existence of unused types

// Macros have to be imported first so they will be fully available in the library
#[cfg(test)]
pub(in crate) mod testing_helpers;

#[macro_use]
extern crate strum_macros;

pub(in crate) mod iterator_utils;
pub(in crate) mod keywords;
pub(in crate) mod types;
