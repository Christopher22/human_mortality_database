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

/// Birth counts indexed by year and sex.
pub type Births<Y> = Table<Y, Empty, covariates::Sex, values::Births>;
/// Death counts indexed by year, age, and sex.
pub type Deaths<Y, A> = Table<Y, A, covariates::Sex, values::Deaths>;
/// Life table rows indexed by year and age.
pub type LifeTable<Y, A> = Table<Y, A, Empty, values::LifeTableRow>;
/// Life expectancy at birth indexed by year and sex.
pub type LifeExpectanciesAtBirth<Y> =
    Table<Y, Empty, covariates::Sex, values::LifeExpectancyAtBirth>;
/// Central death rates indexed by year, age, and sex.
pub type CentralDeathRates<Y, A> = Table<Y, A, covariates::Sex, values::CentralDeathRate>;

pub use self::table::{Country, Empty, Index, Range, Table};
