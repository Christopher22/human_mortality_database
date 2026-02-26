# Query the Human Mortality Database

This (inofficial) crate provides a strongly typed Rust interface for loading and querying tables from the Human Mortality Database (HMD). It parses HMD text files into typed tables and lets you query by year, age, and sex with index types that model scalar and interval dimensions explicitly. 
The table index model is intentionally small and composable. `Single<T>` represents an exact index key such as one year or one age. `Range<T, N>` represents an interval index key such as a five-year or ten-year group, where `N` captures the declared grouping width at the type level. `Empty` represents a missing dimension and is useful for one-dimensional tables or data that does not vary by that axis.

## Installation

Add the crate to your project with Cargo. If you use `cargo add`, run `cargo add human_mortality_database`. If you prefer editing the manifest manually, add `human_mortality_database` to the `[dependencies]` section in your `Cargo.toml`.

## Usage

The example below shows how to load yearly births using a single year index and then query one value.

```rust
use human_mortality_database::covariates::{Sex, Year};
use human_mortality_database::{Births, Single};

let input = "Germany, Births\tLast modified: 03 Jun 2022; Methods Protocol: v6 (2017)\n\nYear Female Male Total\n1990 10 11 21\n";
let table = Births::<Single<Year>>::load(input.as_bytes()).unwrap();
let female_births_1990 = table.query(Year(1990), (), Sex::Female);
assert!(female_births_1990.is_some());
```

The next example shows a grouped-year table where `Range<Year, 10>` is used as the year index type while queries still use a single year value.

```rust
use human_mortality_database::covariates::{Sex, Year};
use human_mortality_database::{LifeExpectanciesAtBirth, Range};

let input = "Germany, Life expectancy at birth (period, 1x10)\tLast modified: 03 Jun 2022; Methods Protocol: v6 (2017)\n\nYear Female Male Total\n1990-1999 80.0 75.0 77.5\n2000-2009 82.0 77.0 79.5\n";
let table = LifeExpectanciesAtBirth::<Range<Year, 10>>::load(input.as_bytes()).unwrap();
let female_2004 = table.query(Year(2004), (), Sex::Female);
assert!(female_2004.is_some());
```
