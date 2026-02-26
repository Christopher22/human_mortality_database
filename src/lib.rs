#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
//! Rust library for querying the Human Mortality Database.

pub mod covariates;
mod table;
pub mod values;

pub use self::table::{Country, Empty, Index, Range, Table};
