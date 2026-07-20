use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use crate::covariates::{Age, Sex, Year};
use crate::values::LifeTableRow;
use crate::{
    Births, CentralDeathRates, Country, Deaths, DownloadableTable, Empty, LifeExpectanciesAtBirth,
    LifeTable, Range, Session, Single, Table,
};

type Year1 = Single<Year>;
type Year5 = Range<Year, 5>;
type Year10 = Range<Year, 10>;
type Age1 = Single<Age>;
type Age5 = Range<Age, 5>;
type Age10 = Range<Age, 10>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableKind {
    Births,
    Deaths,
    LifeTable,
    LifeExpectancyAtBirth,
    CentralDeathRate,
}

impl TableKind {
    fn from_python(value: PyTableKind) -> Self {
        match value {
            PyTableKind::Births => Self::Births,
            PyTableKind::Deaths => Self::Deaths,
            PyTableKind::LifeTable => Self::LifeTable,
            PyTableKind::LifeExpectancyAtBirth => Self::LifeExpectancyAtBirth,
            PyTableKind::CentralDeathRate => Self::CentralDeathRate,
        }
    }
}

#[pyclass(name = "TableKind", eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PyTableKind {
    Births,
    Deaths,
    LifeTable,
    LifeExpectancyAtBirth,
    CentralDeathRate,
}

#[pyclass(name = "Sex", eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PySex {
    Female,
    Male,
}

#[pyclass(name = "Country", eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PyCountry {
    Australia,
    Austria,
    Belarus,
    Belgium,
    Bulgaria,
    Canada,
    Chile,
    Croatia,
    Czechia,
    Denmark,
    Estonia,
    Finland,
    FranceTotalPopulation,
    FranceCivilianPopulation,
    Germany,
    EastGermany,
    WestGermany,
    Greece,
    HongKong,
    Hungary,
    Iceland,
    Ireland,
    Israel,
    Italy,
    Japan,
    Latvia,
    Lithuania,
    Luxembourg,
    Netherlands,
    NewZealandTotalPopulation,
    NewZealandMaori,
    NewZealandNonMaori,
    Norway,
    Poland,
    Portugal,
    RepublicOfKorea,
    Russia,
    Slovakia,
    Slovenia,
    Spain,
    Sweden,
    Switzerland,
    Taiwan,
    UnitedKingdomTotalPopulation,
    EnglandAndWalesTotalPopulation,
    EnglandAndWalesCivilianPopulation,
    Scotland,
    NorthernIreland,
    UnitedStatesOfAmerica,
    Ukraine,
}

impl From<PyCountry> for Country {
    fn from(value: PyCountry) -> Self {
        match value {
            PyCountry::Australia => Country::Australia,
            PyCountry::Austria => Country::Austria,
            PyCountry::Belarus => Country::Belarus,
            PyCountry::Belgium => Country::Belgium,
            PyCountry::Bulgaria => Country::Bulgaria,
            PyCountry::Canada => Country::Canada,
            PyCountry::Chile => Country::Chile,
            PyCountry::Croatia => Country::Croatia,
            PyCountry::Czechia => Country::Czechia,
            PyCountry::Denmark => Country::Denmark,
            PyCountry::Estonia => Country::Estonia,
            PyCountry::Finland => Country::Finland,
            PyCountry::FranceTotalPopulation => Country::FranceTotalPopulation,
            PyCountry::FranceCivilianPopulation => Country::FranceCivilianPopulation,
            PyCountry::Germany => Country::Germany,
            PyCountry::EastGermany => Country::EastGermany,
            PyCountry::WestGermany => Country::WestGermany,
            PyCountry::Greece => Country::Greece,
            PyCountry::HongKong => Country::HongKong,
            PyCountry::Hungary => Country::Hungary,
            PyCountry::Iceland => Country::Iceland,
            PyCountry::Ireland => Country::Ireland,
            PyCountry::Israel => Country::Israel,
            PyCountry::Italy => Country::Italy,
            PyCountry::Japan => Country::Japan,
            PyCountry::Latvia => Country::Latvia,
            PyCountry::Lithuania => Country::Lithuania,
            PyCountry::Luxembourg => Country::Luxembourg,
            PyCountry::Netherlands => Country::Netherlands,
            PyCountry::NewZealandTotalPopulation => Country::NewZealandTotalPopulation,
            PyCountry::NewZealandMaori => Country::NewZealandMaori,
            PyCountry::NewZealandNonMaori => Country::NewZealandNonMaori,
            PyCountry::Norway => Country::Norway,
            PyCountry::Poland => Country::Poland,
            PyCountry::Portugal => Country::Portugal,
            PyCountry::RepublicOfKorea => Country::RepublicOfKorea,
            PyCountry::Russia => Country::Russia,
            PyCountry::Slovakia => Country::Slovakia,
            PyCountry::Slovenia => Country::Slovenia,
            PyCountry::Spain => Country::Spain,
            PyCountry::Sweden => Country::Sweden,
            PyCountry::Switzerland => Country::Switzerland,
            PyCountry::Taiwan => Country::Taiwan,
            PyCountry::UnitedKingdomTotalPopulation => Country::UnitedKingdomTotalPopulation,
            PyCountry::EnglandAndWalesTotalPopulation => Country::EnglandAndWalesTotalPopulation,
            PyCountry::EnglandAndWalesCivilianPopulation => {
                Country::EnglandAndWalesCivilianPopulation
            }
            PyCountry::Scotland => Country::Scotland,
            PyCountry::NorthernIreland => Country::NorthernIreland,
            PyCountry::UnitedStatesOfAmerica => Country::UnitedStatesOfAmerica,
            PyCountry::Ukraine => Country::Ukraine,
        }
    }
}

#[derive(Debug)]
enum TableHandle {
    Births1(Births<Year1>),
    Births5(Births<Year5>),
    Births10(Births<Year10>),

    Deaths11(Deaths<Year1, Age1>),
    Deaths15(Deaths<Year1, Age5>),
    Deaths110(Deaths<Year1, Age10>),
    Deaths51(Deaths<Year5, Age1>),
    Deaths55(Deaths<Year5, Age5>),
    Deaths510(Deaths<Year5, Age10>),
    Deaths101(Deaths<Year10, Age1>),
    Deaths105(Deaths<Year10, Age5>),
    Deaths1010(Deaths<Year10, Age10>),

    LifeTable11(LifeTable<Year1, Age1>),
    LifeTable15(LifeTable<Year1, Age5>),
    LifeTable110(LifeTable<Year1, Age10>),
    LifeTable51(LifeTable<Year5, Age1>),
    LifeTable55(LifeTable<Year5, Age5>),
    LifeTable510(LifeTable<Year5, Age10>),
    LifeTable101(LifeTable<Year10, Age1>),
    LifeTable105(LifeTable<Year10, Age5>),
    LifeTable1010(LifeTable<Year10, Age10>),

    LifeExpectancy1(LifeExpectanciesAtBirth<Year1>),
    LifeExpectancy5(LifeExpectanciesAtBirth<Year5>),
    LifeExpectancy10(LifeExpectanciesAtBirth<Year10>),

    CentralDeathRate11(CentralDeathRates<Year1, Age1>),
    CentralDeathRate15(CentralDeathRates<Year1, Age5>),
    CentralDeathRate110(CentralDeathRates<Year1, Age10>),
    CentralDeathRate51(CentralDeathRates<Year5, Age1>),
    CentralDeathRate55(CentralDeathRates<Year5, Age5>),
    CentralDeathRate510(CentralDeathRates<Year5, Age10>),
    CentralDeathRate101(CentralDeathRates<Year10, Age1>),
    CentralDeathRate105(CentralDeathRates<Year10, Age5>),
    CentralDeathRate1010(CentralDeathRates<Year10, Age10>),
}

impl TableHandle {
    fn country(&self) -> Country {
        match self {
            Self::Births1(table) => table.country,
            Self::Births5(table) => table.country,
            Self::Births10(table) => table.country,
            Self::Deaths11(table) => table.country,
            Self::Deaths15(table) => table.country,
            Self::Deaths110(table) => table.country,
            Self::Deaths51(table) => table.country,
            Self::Deaths55(table) => table.country,
            Self::Deaths510(table) => table.country,
            Self::Deaths101(table) => table.country,
            Self::Deaths105(table) => table.country,
            Self::Deaths1010(table) => table.country,
            Self::LifeTable11(table) => table.country,
            Self::LifeTable15(table) => table.country,
            Self::LifeTable110(table) => table.country,
            Self::LifeTable51(table) => table.country,
            Self::LifeTable55(table) => table.country,
            Self::LifeTable510(table) => table.country,
            Self::LifeTable101(table) => table.country,
            Self::LifeTable105(table) => table.country,
            Self::LifeTable1010(table) => table.country,
            Self::LifeExpectancy1(table) => table.country,
            Self::LifeExpectancy5(table) => table.country,
            Self::LifeExpectancy10(table) => table.country,
            Self::CentralDeathRate11(table) => table.country,
            Self::CentralDeathRate15(table) => table.country,
            Self::CentralDeathRate110(table) => table.country,
            Self::CentralDeathRate51(table) => table.country,
            Self::CentralDeathRate55(table) => table.country,
            Self::CentralDeathRate510(table) => table.country,
            Self::CentralDeathRate101(table) => table.country,
            Self::CentralDeathRate105(table) => table.country,
            Self::CentralDeathRate1010(table) => table.country,
        }
    }

    fn last_modified(&self) -> chrono::NaiveDate {
        match self {
            Self::Births1(table) => table.last_modified,
            Self::Births5(table) => table.last_modified,
            Self::Births10(table) => table.last_modified,
            Self::Deaths11(table) => table.last_modified,
            Self::Deaths15(table) => table.last_modified,
            Self::Deaths110(table) => table.last_modified,
            Self::Deaths51(table) => table.last_modified,
            Self::Deaths55(table) => table.last_modified,
            Self::Deaths510(table) => table.last_modified,
            Self::Deaths101(table) => table.last_modified,
            Self::Deaths105(table) => table.last_modified,
            Self::Deaths1010(table) => table.last_modified,
            Self::LifeTable11(table) => table.last_modified,
            Self::LifeTable15(table) => table.last_modified,
            Self::LifeTable110(table) => table.last_modified,
            Self::LifeTable51(table) => table.last_modified,
            Self::LifeTable55(table) => table.last_modified,
            Self::LifeTable510(table) => table.last_modified,
            Self::LifeTable101(table) => table.last_modified,
            Self::LifeTable105(table) => table.last_modified,
            Self::LifeTable1010(table) => table.last_modified,
            Self::LifeExpectancy1(table) => table.last_modified,
            Self::LifeExpectancy5(table) => table.last_modified,
            Self::LifeExpectancy10(table) => table.last_modified,
            Self::CentralDeathRate11(table) => table.last_modified,
            Self::CentralDeathRate15(table) => table.last_modified,
            Self::CentralDeathRate110(table) => table.last_modified,
            Self::CentralDeathRate51(table) => table.last_modified,
            Self::CentralDeathRate55(table) => table.last_modified,
            Self::CentralDeathRate510(table) => table.last_modified,
            Self::CentralDeathRate101(table) => table.last_modified,
            Self::CentralDeathRate105(table) => table.last_modified,
            Self::CentralDeathRate1010(table) => table.last_modified,
        }
    }

    fn query_scalar(
        &self,
        year: u16,
        age: Option<u8>,
        sex: Option<PySex>,
    ) -> PyResult<Option<f64>> {
        match self {
            Self::Births1(table) => query_year_sex_scalar(table, year, sex),
            Self::Births5(table) => query_year_sex_scalar(table, year, sex),
            Self::Births10(table) => query_year_sex_scalar(table, year, sex),

            Self::Deaths11(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::Deaths15(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::Deaths110(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::Deaths51(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::Deaths55(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::Deaths510(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::Deaths101(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::Deaths105(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::Deaths1010(table) => query_year_age_sex_scalar(table, year, age, sex),

            Self::LifeTable11(_)
            | Self::LifeTable15(_)
            | Self::LifeTable110(_)
            | Self::LifeTable51(_)
            | Self::LifeTable55(_)
            | Self::LifeTable510(_)
            | Self::LifeTable101(_)
            | Self::LifeTable105(_)
            | Self::LifeTable1010(_) => Err(PyValueError::new_err(
                "life_table rows are not scalar values; use query_life_table_row(year, age)",
            )),

            Self::LifeExpectancy1(table) => query_year_sex_scalar(table, year, sex),
            Self::LifeExpectancy5(table) => query_year_sex_scalar(table, year, sex),
            Self::LifeExpectancy10(table) => query_year_sex_scalar(table, year, sex),

            Self::CentralDeathRate11(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::CentralDeathRate15(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::CentralDeathRate110(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::CentralDeathRate51(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::CentralDeathRate55(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::CentralDeathRate510(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::CentralDeathRate101(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::CentralDeathRate105(table) => query_year_age_sex_scalar(table, year, age, sex),
            Self::CentralDeathRate1010(table) => query_year_age_sex_scalar(table, year, age, sex),
        }
    }

    fn query_life_table_row(&self, year: u16, age: u8) -> PyResult<Option<PyLifeTableRow>> {
        match self {
            Self::LifeTable11(table) => query_life_table_row(table, year, age),
            Self::LifeTable15(table) => query_life_table_row(table, year, age),
            Self::LifeTable110(table) => query_life_table_row(table, year, age),
            Self::LifeTable51(table) => query_life_table_row(table, year, age),
            Self::LifeTable55(table) => query_life_table_row(table, year, age),
            Self::LifeTable510(table) => query_life_table_row(table, year, age),
            Self::LifeTable101(table) => query_life_table_row(table, year, age),
            Self::LifeTable105(table) => query_life_table_row(table, year, age),
            Self::LifeTable1010(table) => query_life_table_row(table, year, age),
            _ => Err(PyValueError::new_err(
                "query_life_table_row is only available for life_table tables",
            )),
        }
    }
}

#[pyclass(name = "Session")]
#[derive(Debug)]
struct PySession {
    inner: Session,
}

#[pymethods]
impl PySession {
    #[staticmethod]
    fn login(username: &str, password: &str) -> PyResult<Self> {
        let inner = Session::login(username.to_owned(), password.to_owned())
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(Self { inner })
    }

    #[pyo3(signature = (table, country, year_interval=1, age_interval=1))]
    fn download(
        &self,
        table: PyTableKind,
        country: PyCountry,
        year_interval: usize,
        age_interval: usize,
    ) -> PyResult<PyTable> {
        let kind = TableKind::from_python(table);
        let country = Country::from(country);
        let inner = download_table(&self.inner, kind, country, year_interval, age_interval)?;
        Ok(PyTable { inner })
    }
}

#[pyclass(name = "LifeTableRow", frozen)]
#[derive(Debug, Clone, Copy)]
struct PyLifeTableRow {
    #[pyo3(get)]
    mx: f64,
    #[pyo3(get)]
    qx: f64,
    #[pyo3(get)]
    ax: f64,
    #[pyo3(get)]
    lx: f64,
    #[pyo3(get)]
    dx: f64,
    #[pyo3(get)]
    lx_person_years: f64,
    #[pyo3(get)]
    tx: f64,
    #[pyo3(get)]
    ex: f64,
}

impl From<LifeTableRow> for PyLifeTableRow {
    fn from(value: LifeTableRow) -> Self {
        Self {
            mx: f64::from(value.mx),
            qx: f64::from(value.qx),
            ax: f64::from(value.ax),
            lx: f64::from(value.lx),
            dx: f64::from(value.dx),
            lx_person_years: f64::from(value.lx_person_years),
            tx: f64::from(value.tx),
            ex: f64::from(value.ex),
        }
    }
}

#[pyclass(name = "Table")]
#[derive(Debug)]
struct PyTable {
    inner: TableHandle,
}

#[pymethods]
impl PyTable {
    #[getter]
    fn country_code(&self) -> &'static str {
        self.inner.country().code()
    }

    #[getter]
    fn last_modified(&self) -> String {
        self.inner.last_modified().format("%Y-%m-%d").to_string()
    }

    #[pyo3(signature = (year, age=None, sex=None))]
    fn query_scalar(
        &self,
        year: u16,
        age: Option<u8>,
        sex: Option<PySex>,
    ) -> PyResult<Option<f64>> {
        self.inner.query_scalar(year, age, sex)
    }

    fn query_life_table_row(&self, year: u16, age: u8) -> PyResult<Option<PyLifeTableRow>> {
        self.inner.query_life_table_row(year, age)
    }
}

#[pyfunction(signature = (table, data, year_interval=1, age_interval=1))]
fn load_table(
    table: PyTableKind,
    data: &[u8],
    year_interval: usize,
    age_interval: usize,
) -> PyResult<PyTable> {
    let kind = TableKind::from_python(table);
    let inner = load_table_from_bytes(kind, data, year_interval, age_interval)?;
    Ok(PyTable { inner })
}

fn parse_sex(sex: Option<PySex>) -> PyResult<Sex> {
    match sex {
        Some(PySex::Female) => Ok(Sex::Female),
        Some(PySex::Male) => Ok(Sex::Male),
        None => Err(PyValueError::new_err("missing required sex argument")),
    }
}

fn parse_age(age: Option<u8>) -> PyResult<Age> {
    let value = age.ok_or_else(|| PyValueError::new_err("missing required age argument"))?;
    Age::try_from(value).map_err(|error| PyValueError::new_err(error.to_string()))
}

fn parse_interval(name: &str, value: usize) -> PyResult<usize> {
    match value {
        1 | 5 | 10 => Ok(value),
        _ => Err(PyValueError::new_err(format!(
            "unsupported {name}_interval={value}. expected one of: 1, 5, 10"
        ))),
    }
}

fn query_year_sex_scalar<Y, D>(
    table: &Table<Y, Empty, Sex, Option<D>>,
    year: u16,
    sex: Option<PySex>,
) -> PyResult<Option<f64>>
where
    Y: crate::table::Index<Value = Year>,
    D: Copy,
    f64: From<D>,
{
    let sex = parse_sex(sex)?;
    Ok(table
        .query(Year(year), (), sex)
        .copied()
        .flatten()
        .map(f64::from))
}

fn query_year_age_sex_scalar<Y, A, D>(
    table: &Table<Y, A, Sex, Option<D>>,
    year: u16,
    age: Option<u8>,
    sex: Option<PySex>,
) -> PyResult<Option<f64>>
where
    Y: crate::table::Index<Value = Year>,
    A: crate::table::Index<Value = Age>,
    D: Copy,
    f64: From<D>,
{
    let age = parse_age(age)?;
    let sex = parse_sex(sex)?;
    Ok(table
        .query(Year(year), age, sex)
        .copied()
        .flatten()
        .map(f64::from))
}

fn query_life_table_row<Y, A>(
    table: &Table<Y, A, Empty, Option<LifeTableRow>>,
    year: u16,
    age: u8,
) -> PyResult<Option<PyLifeTableRow>>
where
    Y: crate::table::Index<Value = Year>,
    A: crate::table::Index<Value = Age>,
{
    let age = Age::try_from(age).map_err(|error| PyValueError::new_err(error.to_string()))?;
    Ok(table
        .query(Year(year), age, ())
        .copied()
        .flatten()
        .map(PyLifeTableRow::from))
}

fn download_typed<T: DownloadableTable>(session: &Session, country: Country) -> PyResult<T> {
    session
        .download(country)
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))
}

fn load_births(data: &[u8], year_interval: usize) -> PyResult<TableHandle> {
    match parse_interval("year", year_interval)? {
        1 => Births::<Year1>::load(data)
            .map(TableHandle::Births1)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        5 => Births::<Year5>::load(data)
            .map(TableHandle::Births5)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        10 => Births::<Year10>::load(data)
            .map(TableHandle::Births10)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        _ => unreachable!(),
    }
}

fn load_deaths(data: &[u8], year_interval: usize, age_interval: usize) -> PyResult<TableHandle> {
    let year_interval = parse_interval("year", year_interval)?;
    let age_interval = parse_interval("age", age_interval)?;

    match (year_interval, age_interval) {
        (1, 1) => Deaths::<Year1, Age1>::load(data)
            .map(TableHandle::Deaths11)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (1, 5) => Deaths::<Year1, Age5>::load(data)
            .map(TableHandle::Deaths15)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (1, 10) => Deaths::<Year1, Age10>::load(data)
            .map(TableHandle::Deaths110)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (5, 1) => Deaths::<Year5, Age1>::load(data)
            .map(TableHandle::Deaths51)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (5, 5) => Deaths::<Year5, Age5>::load(data)
            .map(TableHandle::Deaths55)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (5, 10) => Deaths::<Year5, Age10>::load(data)
            .map(TableHandle::Deaths510)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (10, 1) => Deaths::<Year10, Age1>::load(data)
            .map(TableHandle::Deaths101)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (10, 5) => Deaths::<Year10, Age5>::load(data)
            .map(TableHandle::Deaths105)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (10, 10) => Deaths::<Year10, Age10>::load(data)
            .map(TableHandle::Deaths1010)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        _ => unreachable!(),
    }
}

fn load_life_table(
    data: &[u8],
    year_interval: usize,
    age_interval: usize,
) -> PyResult<TableHandle> {
    let year_interval = parse_interval("year", year_interval)?;
    let age_interval = parse_interval("age", age_interval)?;

    match (year_interval, age_interval) {
        (1, 1) => LifeTable::<Year1, Age1>::load(data)
            .map(TableHandle::LifeTable11)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (1, 5) => LifeTable::<Year1, Age5>::load(data)
            .map(TableHandle::LifeTable15)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (1, 10) => LifeTable::<Year1, Age10>::load(data)
            .map(TableHandle::LifeTable110)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (5, 1) => LifeTable::<Year5, Age1>::load(data)
            .map(TableHandle::LifeTable51)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (5, 5) => LifeTable::<Year5, Age5>::load(data)
            .map(TableHandle::LifeTable55)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (5, 10) => LifeTable::<Year5, Age10>::load(data)
            .map(TableHandle::LifeTable510)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (10, 1) => LifeTable::<Year10, Age1>::load(data)
            .map(TableHandle::LifeTable101)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (10, 5) => LifeTable::<Year10, Age5>::load(data)
            .map(TableHandle::LifeTable105)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (10, 10) => LifeTable::<Year10, Age10>::load(data)
            .map(TableHandle::LifeTable1010)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        _ => unreachable!(),
    }
}

fn load_life_expectancy(data: &[u8], year_interval: usize) -> PyResult<TableHandle> {
    match parse_interval("year", year_interval)? {
        1 => LifeExpectanciesAtBirth::<Year1>::load(data)
            .map(TableHandle::LifeExpectancy1)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        5 => LifeExpectanciesAtBirth::<Year5>::load(data)
            .map(TableHandle::LifeExpectancy5)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        10 => LifeExpectanciesAtBirth::<Year10>::load(data)
            .map(TableHandle::LifeExpectancy10)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        _ => unreachable!(),
    }
}

fn load_central_death_rate(
    data: &[u8],
    year_interval: usize,
    age_interval: usize,
) -> PyResult<TableHandle> {
    let year_interval = parse_interval("year", year_interval)?;
    let age_interval = parse_interval("age", age_interval)?;

    match (year_interval, age_interval) {
        (1, 1) => CentralDeathRates::<Year1, Age1>::load(data)
            .map(TableHandle::CentralDeathRate11)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (1, 5) => CentralDeathRates::<Year1, Age5>::load(data)
            .map(TableHandle::CentralDeathRate15)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (1, 10) => CentralDeathRates::<Year1, Age10>::load(data)
            .map(TableHandle::CentralDeathRate110)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (5, 1) => CentralDeathRates::<Year5, Age1>::load(data)
            .map(TableHandle::CentralDeathRate51)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (5, 5) => CentralDeathRates::<Year5, Age5>::load(data)
            .map(TableHandle::CentralDeathRate55)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (5, 10) => CentralDeathRates::<Year5, Age10>::load(data)
            .map(TableHandle::CentralDeathRate510)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (10, 1) => CentralDeathRates::<Year10, Age1>::load(data)
            .map(TableHandle::CentralDeathRate101)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (10, 5) => CentralDeathRates::<Year10, Age5>::load(data)
            .map(TableHandle::CentralDeathRate105)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        (10, 10) => CentralDeathRates::<Year10, Age10>::load(data)
            .map(TableHandle::CentralDeathRate1010)
            .map_err(|error| PyRuntimeError::new_err(error.to_string())),
        _ => unreachable!(),
    }
}

fn download_table(
    session: &Session,
    kind: TableKind,
    country: Country,
    year_interval: usize,
    age_interval: usize,
) -> PyResult<TableHandle> {
    match kind {
        TableKind::Births => {
            if age_interval != 1 {
                return Err(PyValueError::new_err(
                    "births does not use age_interval; pass age_interval=1",
                ));
            }

            match parse_interval("year", year_interval)? {
                1 => download_typed::<Births<Year1>>(session, country).map(TableHandle::Births1),
                5 => download_typed::<Births<Year5>>(session, country).map(TableHandle::Births5),
                10 => download_typed::<Births<Year10>>(session, country).map(TableHandle::Births10),
                _ => unreachable!(),
            }
        }
        TableKind::Deaths => {
            let year_interval = parse_interval("year", year_interval)?;
            let age_interval = parse_interval("age", age_interval)?;
            match (year_interval, age_interval) {
                (1, 1) => download_typed::<Deaths<Year1, Age1>>(session, country)
                    .map(TableHandle::Deaths11),
                (1, 5) => download_typed::<Deaths<Year1, Age5>>(session, country)
                    .map(TableHandle::Deaths15),
                (1, 10) => download_typed::<Deaths<Year1, Age10>>(session, country)
                    .map(TableHandle::Deaths110),
                (5, 1) => download_typed::<Deaths<Year5, Age1>>(session, country)
                    .map(TableHandle::Deaths51),
                (5, 5) => download_typed::<Deaths<Year5, Age5>>(session, country)
                    .map(TableHandle::Deaths55),
                (5, 10) => download_typed::<Deaths<Year5, Age10>>(session, country)
                    .map(TableHandle::Deaths510),
                (10, 1) => download_typed::<Deaths<Year10, Age1>>(session, country)
                    .map(TableHandle::Deaths101),
                (10, 5) => download_typed::<Deaths<Year10, Age5>>(session, country)
                    .map(TableHandle::Deaths105),
                (10, 10) => download_typed::<Deaths<Year10, Age10>>(session, country)
                    .map(TableHandle::Deaths1010),
                _ => unreachable!(),
            }
        }
        TableKind::LifeTable => {
            let year_interval = parse_interval("year", year_interval)?;
            let age_interval = parse_interval("age", age_interval)?;
            match (year_interval, age_interval) {
                (1, 1) => download_typed::<LifeTable<Year1, Age1>>(session, country)
                    .map(TableHandle::LifeTable11),
                (1, 5) => download_typed::<LifeTable<Year1, Age5>>(session, country)
                    .map(TableHandle::LifeTable15),
                (1, 10) => download_typed::<LifeTable<Year1, Age10>>(session, country)
                    .map(TableHandle::LifeTable110),
                (5, 1) => download_typed::<LifeTable<Year5, Age1>>(session, country)
                    .map(TableHandle::LifeTable51),
                (5, 5) => download_typed::<LifeTable<Year5, Age5>>(session, country)
                    .map(TableHandle::LifeTable55),
                (5, 10) => download_typed::<LifeTable<Year5, Age10>>(session, country)
                    .map(TableHandle::LifeTable510),
                (10, 1) => download_typed::<LifeTable<Year10, Age1>>(session, country)
                    .map(TableHandle::LifeTable101),
                (10, 5) => download_typed::<LifeTable<Year10, Age5>>(session, country)
                    .map(TableHandle::LifeTable105),
                (10, 10) => download_typed::<LifeTable<Year10, Age10>>(session, country)
                    .map(TableHandle::LifeTable1010),
                _ => unreachable!(),
            }
        }
        TableKind::LifeExpectancyAtBirth => {
            if age_interval != 1 {
                return Err(PyValueError::new_err(
                    "life_expectancy_at_birth does not use age_interval; pass age_interval=1",
                ));
            }

            match parse_interval("year", year_interval)? {
                1 => download_typed::<LifeExpectanciesAtBirth<Year1>>(session, country)
                    .map(TableHandle::LifeExpectancy1),
                5 => download_typed::<LifeExpectanciesAtBirth<Year5>>(session, country)
                    .map(TableHandle::LifeExpectancy5),
                10 => download_typed::<LifeExpectanciesAtBirth<Year10>>(session, country)
                    .map(TableHandle::LifeExpectancy10),
                _ => unreachable!(),
            }
        }
        TableKind::CentralDeathRate => {
            let year_interval = parse_interval("year", year_interval)?;
            let age_interval = parse_interval("age", age_interval)?;
            match (year_interval, age_interval) {
                (1, 1) => download_typed::<CentralDeathRates<Year1, Age1>>(session, country)
                    .map(TableHandle::CentralDeathRate11),
                (1, 5) => download_typed::<CentralDeathRates<Year1, Age5>>(session, country)
                    .map(TableHandle::CentralDeathRate15),
                (1, 10) => download_typed::<CentralDeathRates<Year1, Age10>>(session, country)
                    .map(TableHandle::CentralDeathRate110),
                (5, 1) => download_typed::<CentralDeathRates<Year5, Age1>>(session, country)
                    .map(TableHandle::CentralDeathRate51),
                (5, 5) => download_typed::<CentralDeathRates<Year5, Age5>>(session, country)
                    .map(TableHandle::CentralDeathRate55),
                (5, 10) => download_typed::<CentralDeathRates<Year5, Age10>>(session, country)
                    .map(TableHandle::CentralDeathRate510),
                (10, 1) => download_typed::<CentralDeathRates<Year10, Age1>>(session, country)
                    .map(TableHandle::CentralDeathRate101),
                (10, 5) => download_typed::<CentralDeathRates<Year10, Age5>>(session, country)
                    .map(TableHandle::CentralDeathRate105),
                (10, 10) => download_typed::<CentralDeathRates<Year10, Age10>>(session, country)
                    .map(TableHandle::CentralDeathRate1010),
                _ => unreachable!(),
            }
        }
    }
}

fn load_table_from_bytes(
    kind: TableKind,
    data: &[u8],
    year_interval: usize,
    age_interval: usize,
) -> PyResult<TableHandle> {
    match kind {
        TableKind::Births => {
            if age_interval != 1 {
                return Err(PyValueError::new_err(
                    "births does not use age_interval; pass age_interval=1",
                ));
            }
            load_births(data, year_interval)
        }
        TableKind::Deaths => load_deaths(data, year_interval, age_interval),
        TableKind::LifeTable => load_life_table(data, year_interval, age_interval),
        TableKind::LifeExpectancyAtBirth => {
            if age_interval != 1 {
                return Err(PyValueError::new_err(
                    "life_expectancy_at_birth does not use age_interval; pass age_interval=1",
                ));
            }
            load_life_expectancy(data, year_interval)
        }
        TableKind::CentralDeathRate => load_central_death_rate(data, year_interval, age_interval),
    }
}

fn register_binding_module(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTableKind>()?;
    m.add_class::<PySex>()?;
    m.add_class::<PyCountry>()?;
    m.add_class::<PySession>()?;
    m.add_class::<PyTable>()?;
    m.add_class::<PyLifeTableRow>()?;
    m.add_function(wrap_pyfunction!(load_table, m)?)?;
    Ok(())
}

#[pymodule]
fn human_mortality_database(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_binding_module(py, m)
}
