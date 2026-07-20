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

#[cfg(feature = "python")]
mod binding;
pub mod covariates;
mod download;
mod table;
pub mod values;

/// Birth counts indexed by year and sex.
///
/// The value is `None` where the HMD source data marks a cell as undefined (a lone "." token),
/// which happens for historical gaps in a country's data (e.g. New Zealand Maori births before
/// 1948).
pub type Births<Y> = Table<Y, Empty, covariates::Sex, Option<values::Births>>;
/// Death counts indexed by year, age, and sex.
///
/// The value is `None` where the HMD source data marks a cell as undefined (a lone "." token),
/// which happens for historical gaps in a country's data (e.g. Belgium during WWI).
pub type Deaths<Y, A> = Table<Y, A, covariates::Sex, Option<values::Deaths>>;
/// Life table rows indexed by year and age, and optionally by sex.
///
/// The row is `None` where the HMD source data marks the entire row as undefined (a lone "."
/// token in every column), which happens for historical gaps in a country's data (e.g. Belgium
/// during WWI).
pub type LifeTable<Y, A, S = Empty> = Table<Y, A, S, Option<values::LifeTableRow>>;
/// Life expectancy at birth indexed by year and sex.
///
/// The value is `None` where the HMD source data marks a cell as undefined (a lone "." token),
/// which happens for historical gaps in a country's data (e.g. Belgium during WWI).
pub type LifeExpectanciesAtBirth<Y> =
    Table<Y, Empty, covariates::Sex, Option<values::LifeExpectancyAtBirth>>;
/// Central death rates indexed by year, age, and sex.
///
/// The value is `None` where the HMD source data marks the rate as undefined (a lone "."
/// token), which commonly happens for the open-ended terminal age group when the corresponding
/// sex recorded zero exposure for that year, or during historical gaps in a country's data.
pub type CentralDeathRates<Y, A> = Table<Y, A, covariates::Sex, Option<values::CentralDeathRate>>;

pub use self::download::{DownloadableTable, Error as DownloadError, Session};
pub use self::table::{Country, Empty, Index, Range, Single, Table};
