# Query the Human Mortality Database

This (inofficial) crate provides a strongly typed Rust interface for loading and querying tables from the [Human Mortality Database (HMD)](https://www.mortality.org/). It parses HMD text files into typed tables and lets you query by year, age, and sex with index types that model scalar and interval dimensions explicitly. 
The table index model is intentionally small and composable. `Single<T>` represents an exact index key such as one year or one age. `Range<T, N>` represents an interval index key such as a five-year or ten-year group, where `N` captures the declared grouping width at the type level. `Empty` represents a missing dimension and is useful for one-dimensional tables or data that does not vary by that axis.

## Installation

Add the crate to your project with Cargo. If you use `cargo add`, run `cargo add human_mortality_database`. If you prefer editing the manifest manually, add `human_mortality_database` to the `[dependencies]` section in your `Cargo.toml`.

### Building the Python binding
This crate exposes optional Python bindings backed by the same Rust core logic. If you want to use it, use `maturin` to build/install the extension module:

```bash
pip install maturin
maturin develop --release --features python
```

The Python module name is `human_mortality_database`.

## Usage

### Rust
The example below shows how to login, download yearly births, and query one value.

```rust
use human_mortality_database::covariates::{Sex, Year};
use human_mortality_database::{Births, Country, Session, Single};

let session = Session::login("HMD_USERNAME", "HMD_PASSWORD").unwrap();
let table: Births<Single<Year>> = session
	.download(Country::Germany)
	.unwrap();

let female_births_1990 = table[(Year(1990), Sex::Female)];
```

The next example shows a grouped-year table where `Range<Year, 10>` is used as the year index type while queries still use a single year value.

```rust
use human_mortality_database::covariates::{Sex, Year};
use human_mortality_database::{Country, LifeExpectanciesAtBirth, Range, Session};

let session = Session::login("HMD_USERNAME", "HMD_PASSWORD").unwrap();
let table: LifeExpectanciesAtBirth<Range<Year, 10>> = session
	.download(Country::Germany)
	.unwrap();

let female_2004 = table[(Year(2004), Sex::Female)];
```

### Python bindings
The Python binding wraps the extensive generics of the Rust codebase. For downloads, use values from `Country` (e.g. `Country.Germany`, `Country.UnitedStatesOfAmerica`). Supported interval sizes are `1`, `5`, and `10`. 

```python
from human_mortality_database import Country, Session, Sex, TableKind, load_table

# Option 1: download directly from HMD
session = Session.login("HMD_USERNAME", "HMD_PASSWORD")
table = session.download(TableKind.Deaths, Country.Germany, year_interval=1, age_interval=5)
value = table.query_scalar(1990, age=2, sex=Sex.Female)

# Option 2: parse existing file bytes
with open("tests/test_data/Mx_1x1.txt", "rb") as f:
	mx = load_table(TableKind.CentralDeathRate, f.read(), year_interval=1, age_interval=1)
print(mx.query_scalar(1990, age=1, sex=Sex.Male))
```

Valid  `TableKind` values are:

- `TableKind.Births`
- `TableKind.Deaths`
- `TableKind.LifeTable`
- `TableKind.LifeExpectancyAtBirth`
- `TableKind.CentralDeathRate`

## Running tests

Download tests require HMD credentials in environment variables. You can specify those i.e. in `.cargo/config.toml` or directly in the shelL:

```bash
export HMD_USERNAME="your-email@example.com"
export HMD_PASSWORD="your-password"
cargo test
```

## Acknowledgements
The code for downloading from the Human Mortality Database is based upon the Python code of [Markus Sauerberg](https://github.com/msauerberg/HMD_download_script).

## License
MIT