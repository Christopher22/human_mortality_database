use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::io::BufRead;

use chrono::NaiveDate;

use crate::covariates::{Age, Sex, Year};
use crate::values::{
    AveragePersonYearsLived, Births, CentralDeathRate, Deaths, ExposureToRisk,
    LifeExpectancyAtBirth, LifeTableRow, NumberAlive, NumberDying, PersonYearsLived,
    PopulationSize, ProbabilityOfDying, RemainingLifeExpectancy, TotalPersonYearsLived,
};

/// A trait for types that can be used as indices in the table.
pub trait Index: Copy + Ord {
    /// The type of the index value.
    type Value: Copy + Ord;
    /// The type of the container that holds the values associated with the index.
    type Container<T>;

    /// Finds the value associated with the given index value.
    /// The list of values must be sorted by the index value and its elements should be unique.
    fn find<T>(values: &Self::Container<T>, value: Self::Value) -> Option<&T>;
}

impl Index for Age {
    type Value = Age;
    type Container<T> = Vec<(Self, T)>;

    fn find<T>(values: &Self::Container<T>, value: Self) -> Option<&T> {
        values
            .binary_search_by_key(&value, |(index, _)| *index)
            .ok()
            .map(|index| &values[index].1)
    }
}

impl Index for Year {
    type Value = Year;
    type Container<T> = Vec<(Self, T)>;

    fn find<T>(values: &Self::Container<T>, value: Self) -> Option<&T> {
        values
            .binary_search_by_key(&value, |(index, _)| *index)
            .ok()
            .map(|index| &values[index].1)
    }
}

impl Index for Sex {
    type Value = Self;
    type Container<T> = [(Self, T); 2];

    fn find<T>(values: &Self::Container<T>, value: Self) -> Option<&T> {
        match value {
            Sex::Female => Some(&values[0].1),
            Sex::Male => Some(&values[1].1),
        }
    }
}

/// A range of values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range<T: Index, const N: usize> {
    start: T::Value,
    end: T::Value,
}

impl<T: Index, const N: usize> Range<T, N> {
    /// Check if the range contains the given value.
    pub fn contains(&self, value: T::Value) -> bool {
        self.start <= value && value <= self.end
    }
}

impl<T: Index, const N: usize> PartialOrd for Range<T, N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Index, const N: usize> Ord for Range<T, N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl<T: Index, const N: usize> Index for Range<T, N> {
    type Value = T::Value;
    type Container<I> = Vec<(Self, I)>;

    fn find<U>(values: &Self::Container<U>, value: T::Value) -> Option<&U> {
        values
            .binary_search_by(|(range, _)| {
                if range.contains(value) {
                    Ordering::Equal
                } else if value < range.start {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
            .ok()
            .map(|index| &values[index].1)
    }
}

/// An empty index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Empty;
impl Index for Empty {
    type Value = ();
    type Container<T> = T;

    fn find<T>(values: &T, _value: Self::Value) -> Option<&T> {
        Some(values)
    }
}

/// A country for which data is available in the HMD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Country {
    /// Germany
    Germany,
}

impl Country {
    /// Returns the HMD code for the country.
    pub fn code(&self) -> &'static str {
        match self {
            Country::Germany => "DEUTNP",
        }
    }
}

/// A table of data from the Human Mortality Database.
#[derive(Debug, Clone)]
pub struct Table<Y: Index, A: Index, S: Index, D> {
    /// The country for which the data is available.
    pub country: Country,
    /// The date when the data was last modified.
    pub last_modified: NaiveDate,
    data: Y::Container<A::Container<S::Container<D>>>,
}

impl<Y: Index, A: Index, S: Index, D> Table<Y, A, S, D> {
    /// Try to load a table from the given reader, returning an error if the data is invalid.
    #[allow(private_bounds)]
    pub fn load<R: std::io::Read>(reader: R) -> Result<Self, ImportError>
    where
        Y: ParseIndex + IndexContainer,
        A: ParseIndex + IndexContainer,
        D: ParseData<S>,
    {
        load_impl::<Y, A, S, D, R>(reader)
    }

    /// Querys the table for the given year and age, returning the associated data if found.
    pub fn query(&self, year: Y::Value, age: A::Value, sex: S::Value) -> Option<&D> {
        Y::find(&self.data, year)
            .and_then(|ages| A::find(ages, age))
            .and_then(|sexes| S::find(sexes, sex))
    }
}

fn load_impl<Y, A, S, D, R>(reader: R) -> Result<Table<Y, A, S, D>, ImportError>
where
    Y: Index + ParseIndex + IndexContainer,
    A: Index + ParseIndex + IndexContainer,
    S: Index,
    R: std::io::Read,
    D: ParseData<S>,
{
    let mut non_empty_lines = std::io::BufReader::new(reader)
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ImportError::Io)?
        .into_iter()
        .map(|line| line.trim().to_owned())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if non_empty_lines.is_empty() {
        return Err(ImportError::EmptyInput);
    }

    let metadata = non_empty_lines.remove(0);
    let (country, last_modified) = parse_metadata(&metadata)?;

    if non_empty_lines.is_empty() {
        return Err(ImportError::MissingHeader);
    }

    let header = Header::parse(&non_empty_lines.remove(0));
    let year_index = header.require("year")?;
    let age_index = header.find("age");

    let mut grouped: BTreeMap<Y, BTreeMap<A, S::Container<D>>> = BTreeMap::new();

    for (line_number, line) in non_empty_lines.into_iter().enumerate() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.len() != header.columns.len() {
            return Err(ImportError::MalformedRow {
                line: line_number + 3,
                expected_columns: header.columns.len(),
                actual_columns: fields.len(),
            });
        }

        let year = Y::parse(fields.get(year_index).copied(), "year")?;
        let age = A::parse(age_index.and_then(|idx| fields.get(idx).copied()), "age")?;
        let data = D::parse_data(&header, &fields)?;

        let ages = grouped.entry(year).or_default();
        if ages.insert(age, data).is_some() {
            return Err(ImportError::DuplicateEntry);
        }
    }

    let mut converted_years = BTreeMap::new();
    for (year, age_map) in grouped {
        converted_years.insert(year, A::from_btree(age_map)?);
    }

    Ok(Table {
        country,
        last_modified,
        data: Y::from_btree(converted_years)?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportError {
    /// The input stream cannot be read.
    Io,
    /// The input has no non-empty lines.
    EmptyInput,
    /// The metadata line cannot be parsed.
    InvalidMetadata,
    /// The first line references an unsupported country.
    UnknownCountry,
    /// The metadata does not contain a last-modified date.
    MissingLastModified,
    /// The last-modified date has an invalid format.
    InvalidLastModified,
    /// The column header line is missing.
    MissingHeader,
    /// A required column is missing.
    MissingColumn,
    /// A row does not match the header column count.
    MalformedRow {
        /// 1-based line number in the source text.
        line: usize,
        /// Number of columns expected from the header.
        expected_columns: usize,
        /// Number of columns found in the row.
        actual_columns: usize,
    },
    /// A year value cannot be parsed.
    InvalidYear,
    /// An age value cannot be parsed.
    InvalidAge,
    /// A range token cannot be parsed.
    InvalidRange,
    /// A numeric cell cannot be parsed.
    InvalidNumber,
    /// A semantic value (e.g. negative exposure) is invalid.
    InvalidValue,
    /// The same key combination appears more than once.
    DuplicateEntry,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::Io => write!(f, "failed to read table input"),
            ImportError::EmptyInput => write!(f, "table input is empty"),
            ImportError::InvalidMetadata => write!(f, "invalid metadata line"),
            ImportError::UnknownCountry => write!(f, "unknown country"),
            ImportError::MissingLastModified => write!(f, "missing last modified date"),
            ImportError::InvalidLastModified => write!(f, "invalid last modified date"),
            ImportError::MissingHeader => write!(f, "missing table header"),
            ImportError::MissingColumn => write!(f, "missing required column"),
            ImportError::MalformedRow {
                line,
                expected_columns,
                actual_columns,
            } => write!(
                f,
                "malformed row at line {line}: expected {expected_columns} columns, got {actual_columns}"
            ),
            ImportError::InvalidYear => write!(f, "invalid year value"),
            ImportError::InvalidAge => write!(f, "invalid age value"),
            ImportError::InvalidRange => write!(f, "invalid range value"),
            ImportError::InvalidNumber => write!(f, "invalid numeric value"),
            ImportError::InvalidValue => write!(f, "invalid semantic value"),
            ImportError::DuplicateEntry => write!(f, "duplicate table entry"),
        }
    }
}

impl std::error::Error for ImportError {}

#[derive(Debug, Clone)]
struct Header {
    columns: Vec<String>,
    index: HashMap<String, Vec<usize>>,
}

impl Header {
    fn parse(line: &str) -> Self {
        let columns = line
            .split_whitespace()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let mut index: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, name) in columns.iter().enumerate() {
            index.entry(name.clone()).or_default().push(i);
        }
        Self { columns, index }
    }

    fn find(&self, column: &str) -> Option<usize> {
        if let Some(indices) = self.index.get(column) {
            return indices.first().copied();
        }
        self.columns
            .iter()
            .position(|name| name.eq_ignore_ascii_case(column))
    }

    fn require(&self, column: &str) -> Result<usize, ImportError> {
        self.find(column).ok_or(ImportError::MissingColumn)
    }
}

fn parse_metadata(line: &str) -> Result<(Country, NaiveDate), ImportError> {
    let country_name = line
        .split(',')
        .next()
        .map(str::trim)
        .ok_or(ImportError::InvalidMetadata)?;
    let country = match country_name {
        "Germany" => Country::Germany,
        _ => return Err(ImportError::UnknownCountry),
    };

    let last_modified = line
        .split("Last modified:")
        .nth(1)
        .and_then(|rest| rest.split(';').next())
        .map(str::trim)
        .ok_or(ImportError::MissingLastModified)?;
    let last_modified = NaiveDate::parse_from_str(last_modified, "%d %b %Y")
        .map_err(|_| ImportError::InvalidLastModified)?;
    Ok((country, last_modified))
}

fn parse_numeric_range(token: &str) -> Result<(u16, u16), ImportError> {
    let (start, end) = token.split_once('-').ok_or(ImportError::InvalidRange)?;
    let start = start
        .parse::<u16>()
        .map_err(|_| ImportError::InvalidRange)?;
    let end = end.parse::<u16>().map_err(|_| ImportError::InvalidRange)?;
    if start > end {
        return Err(ImportError::InvalidRange);
    }
    Ok((start, end))
}

fn parse_age_token(token: &str) -> Result<Age, ImportError> {
    if token.contains('-') {
        return Err(ImportError::InvalidAge);
    }
    let token = token.strip_suffix('+').unwrap_or(token);
    let age = token.parse::<u8>().map_err(|_| ImportError::InvalidAge)?;
    Age::try_from(age).map_err(|_| ImportError::InvalidAge)
}

fn parse_required_value<D: ParseScalar>(
    header: &Header,
    fields: &[&str],
    column: &str,
) -> Result<D, ImportError> {
    let index = header.require(column)?;
    D::parse_scalar(fields[index])
}

trait ParseIndex: Index {
    fn parse(token: Option<&str>, field: &str) -> Result<Self, ImportError>;
}

impl ParseIndex for Year {
    fn parse(token: Option<&str>, field: &str) -> Result<Self, ImportError> {
        if field != "year" {
            return Err(ImportError::InvalidYear);
        }
        let token = token.ok_or(ImportError::MissingColumn)?;
        token
            .parse::<u16>()
            .map(Year)
            .map_err(|_| ImportError::InvalidYear)
    }
}

impl<const N: usize> ParseIndex for Range<Year, N> {
    fn parse(token: Option<&str>, field: &str) -> Result<Self, ImportError> {
        if field != "year" {
            return Err(ImportError::InvalidRange);
        }
        parse_numeric_range(token.ok_or(ImportError::MissingColumn)?).map(|(start, end)| {
            let start = Year(start);
            let end = Year(end);
            Self { start, end }
        })
    }
}

impl ParseIndex for Age {
    fn parse(token: Option<&str>, field: &str) -> Result<Self, ImportError> {
        if field != "age" {
            return Err(ImportError::InvalidAge);
        }
        parse_age_token(token.ok_or(ImportError::MissingColumn)?)
    }
}

impl<const N: usize> ParseIndex for Range<Age, N> {
    fn parse(token: Option<&str>, field: &str) -> Result<Self, ImportError> {
        if field != "age" {
            return Err(ImportError::InvalidRange);
        }

        let token = token.ok_or(ImportError::MissingColumn)?;
        if let Some(base) = token.strip_suffix('+') {
            let start = base.parse::<u8>().map_err(|_| ImportError::InvalidAge)?;
            let start = Age::try_from(start).map_err(|_| ImportError::InvalidAge)?;
            return Ok(Self {
                start,
                end: Age::MAX,
            });
        }

        if !token.contains('-') {
            let age = token.parse::<u8>().map_err(|_| ImportError::InvalidAge)?;
            let age = Age::try_from(age).map_err(|_| ImportError::InvalidAge)?;
            return Ok(Self {
                start: age,
                end: age,
            });
        }

        let (start, end) = parse_numeric_range(token)?;
        let start = u8::try_from(start).map_err(|_| ImportError::InvalidAge)?;
        let end = u8::try_from(end).map_err(|_| ImportError::InvalidAge)?;
        let start = Age::try_from(start).map_err(|_| ImportError::InvalidAge)?;
        let end = Age::try_from(end).map_err(|_| ImportError::InvalidAge)?;
        Ok(Self { start, end })
    }
}

impl ParseIndex for Empty {
    fn parse(_token: Option<&str>, _field: &str) -> Result<Self, ImportError> {
        Ok(Empty)
    }
}

trait IndexContainer: Index {
    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError>;
}

impl IndexContainer for Year {
    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError> {
        Ok(map.into_iter().collect())
    }
}

impl IndexContainer for Age {
    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError> {
        Ok(map.into_iter().collect())
    }
}

impl<const N: usize> IndexContainer for Range<Year, N> {
    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError> {
        Ok(map.into_iter().collect())
    }
}

impl<const N: usize> IndexContainer for Range<Age, N> {
    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError> {
        Ok(map.into_iter().collect())
    }
}

impl IndexContainer for Empty {
    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError> {
        map.into_values().next().ok_or(ImportError::EmptyInput)
    }
}

trait ParseData<S: Index>: Sized {
    fn parse_data(header: &Header, fields: &[&str]) -> Result<S::Container<Self>, ImportError>;
}

pub trait ParseScalar: Sized {
    fn parse_scalar(token: &str) -> Result<Self, ImportError>;
}

impl ParseScalar for f64 {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        token.parse::<f64>().map_err(|_| ImportError::InvalidNumber)
    }
}

impl ParseScalar for usize {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        token
            .parse::<usize>()
            .map_err(|_| ImportError::InvalidNumber)
    }
}

impl ParseScalar for PopulationSize {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        token
            .parse::<usize>()
            .map(PopulationSize::from)
            .map_err(|_| ImportError::InvalidNumber)
    }
}

impl ParseScalar for ExposureToRisk {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl ParseScalar for Births {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl ParseScalar for Deaths {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl ParseScalar for LifeExpectancyAtBirth {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl ParseScalar for CentralDeathRate {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl ParseScalar for ProbabilityOfDying {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl ParseScalar for AveragePersonYearsLived {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl ParseScalar for NumberAlive {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl ParseScalar for NumberDying {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl ParseScalar for PersonYearsLived {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl ParseScalar for TotalPersonYearsLived {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl ParseScalar for RemainingLifeExpectancy {
    fn parse_scalar(token: &str) -> Result<Self, ImportError> {
        let value = token
            .parse::<f64>()
            .map_err(|_| ImportError::InvalidNumber)?;
        Self::try_from(value).map_err(|_| ImportError::InvalidValue)
    }
}

impl<D: ParseScalar> ParseData<Sex> for D {
    fn parse_data(
        header: &Header,
        fields: &[&str],
    ) -> Result<<Sex as Index>::Container<Self>, ImportError> {
        let female = header.require("female")?;
        let male = header.require("male")?;
        let female = D::parse_scalar(fields[female])?;
        let male = D::parse_scalar(fields[male])?;
        Ok([(Sex::Female, female), (Sex::Male, male)])
    }
}

impl<D: ParseScalar> ParseData<Empty> for D {
    fn parse_data(
        header: &Header,
        fields: &[&str],
    ) -> Result<<Empty as Index>::Container<Self>, ImportError> {
        if let Some(total) = header.find("total") {
            return D::parse_scalar(fields[total]);
        }

        let data_columns = header
            .columns
            .iter()
            .enumerate()
            .filter_map(|(index, name)| {
                if name.eq_ignore_ascii_case("year") || name.eq_ignore_ascii_case("age") {
                    None
                } else {
                    Some(index)
                }
            })
            .collect::<Vec<_>>();

        if data_columns.len() != 1 {
            return Err(ImportError::MissingColumn);
        }

        D::parse_scalar(fields[data_columns[0]])
    }
}

impl ParseData<Empty> for LifeTableRow {
    fn parse_data(
        header: &Header,
        fields: &[&str],
    ) -> Result<<Empty as Index>::Container<Self>, ImportError> {
        Ok(Self {
            mx: parse_required_value(header, fields, "mx")?,
            qx: parse_required_value(header, fields, "qx")?,
            ax: parse_required_value(header, fields, "ax")?,
            lx: parse_required_value(header, fields, "lx")?,
            dx: parse_required_value(header, fields, "dx")?,
            lx_person_years: parse_required_value(header, fields, "Lx")?,
            tx: parse_required_value(header, fields, "tx")?,
            ex: parse_required_value(header, fields, "ex")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::{Births, Deaths, LifeTableRow};

    #[test]
    fn loads_births_1x1_with_sex_dimension() {
        let input = "Germany,  Births (1-year)\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Female Male Total\n1990 10 11 21\n1991 12 13 25\n";
        let table = Table::<Year, Empty, Sex, Births>::load(input.as_bytes()).unwrap();

        assert_eq!(table.country, Country::Germany);
        assert_eq!(
            table.last_modified,
            NaiveDate::from_ymd_opt(2022, 6, 3).unwrap()
        );
        assert_eq!(
            table
                .query(Year(1990), (), Sex::Female)
                .map(|value| f64::from(*value)),
            Some(10.0)
        );
        assert_eq!(
            table
                .query(Year(1990), (), Sex::Male)
                .map(|value| f64::from(*value)),
            Some(11.0)
        );
        assert_eq!(table.query(Year(1992), (), Sex::Male), None);
    }

    #[test]
    fn loads_mx_1x1_with_open_age_group() {
        let input = "Germany, Death rates (period 1x1),\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Age Female Male Total\n1990 0 0.10 0.20 0.15\n1990 110+ 1.10 1.20 1.15\n";
        let table = Table::<Year, Age, Sex, f64>::load(input.as_bytes()).unwrap();

        assert_eq!(
            table.query(Year(1990), Age::try_from(0).unwrap(), Sex::Female),
            Some(&0.10)
        );
        assert_eq!(
            table.query(Year(1990), Age::try_from(110).unwrap(), Sex::Male),
            Some(&1.20)
        );
    }

    #[test]
    fn loads_deaths_1x5_with_year_ranges() {
        let input = "Germany, Deaths (period 1x5),\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Age Female Male Total\n1990-1994 0 100.0 120.0 220.0\n1995-1999 0 110.0 130.0 240.0\n";
        let table = Table::<Range<Year, 5>, Age, Sex, Deaths>::load(input.as_bytes()).unwrap();

        assert_eq!(
            table
                .query(Year(1992), Age::try_from(0).unwrap(), Sex::Female)
                .map(|value| f64::from(*value)),
            Some(100.0)
        );
        assert_eq!(
            table
                .query(Year(1997), Age::try_from(0).unwrap(), Sex::Male)
                .map(|value| f64::from(*value)),
            Some(130.0)
        );
    }

    #[test]
    fn loads_deaths_5x1_with_age_ranges() {
        let input = "Germany, Deaths (period 5x1),\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Age Female Male Total\n1990 0 100.0 120.0 220.0\n1990 1-4 200.0 220.0 420.0\n1991 1-4 210.0 230.0 440.0\n";
        let table = Table::<Year, Range<Age, 5>, Sex, Deaths>::load(input.as_bytes()).unwrap();

        assert_eq!(
            table
                .query(Year(1990), Age::try_from(2).unwrap(), Sex::Female)
                .map(|value| f64::from(*value)),
            Some(200.0)
        );
        assert_eq!(
            table
                .query(Year(1991), Age::try_from(3).unwrap(), Sex::Male)
                .map(|value| f64::from(*value)),
            Some(230.0)
        );
    }

    #[test]
    fn loads_life_expectancy_1x10_with_year_ranges() {
        let input = "Germany, Life expectancy at birth (period, 1x10)\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Female Male Total\n1990-1999 80.0 75.0 77.5\n2000-2009 82.0 77.0 79.5\n";
        let table =
            Table::<Range<Year, 10>, Empty, Sex, LifeExpectancyAtBirth>::load(input.as_bytes())
                .unwrap();

        assert_eq!(
            table
                .query(Year(2004), (), Sex::Female)
                .map(ToString::to_string),
            Some("82.00".to_owned())
        );
    }

    #[test]
    fn loads_fltper_1x10_life_table_rows() {
        let input = "Germany, Life tables (period 1x10), Females\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Age mx qx ax lx dx Lx Tx ex\n1990-1999 0 0.01 0.02 0.14 100000 2000 99500 7900000 79.0\n1990-1999 1 0.02 0.03 0.50 98000 3000 96500 7800500 78.0\n";
        let table =
            Table::<Range<Year, 10>, Age, Empty, LifeTableRow>::load(input.as_bytes()).unwrap();

        let row = table
            .query(Year(1995), Age::try_from(1).unwrap(), ())
            .copied()
            .unwrap();
        assert_eq!(f64::from(row.ex), 78.0);
        assert_eq!(f64::from(row.mx), 0.02);
    }

    #[test]
    fn loads_bltper_1x1_life_table_rows() {
        let input = "Germany, Life tables (period 1x1), Total\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Age mx qx ax lx dx Lx Tx ex\n1990 0 0.01 0.02 0.14 100000 2000 99500 7900000 79.0\n1991 0 0.02 0.03 0.14 100000 3000 98500 7800500 78.0\n";
        let table = Table::<Year, Age, Empty, LifeTableRow>::load(input.as_bytes()).unwrap();

        assert_eq!(
            table
                .query(Year(1991), Age::try_from(0).unwrap(), ())
                .map(|row| f64::from(row.ex)),
            Some(78.0)
        );
    }

    #[test]
    fn parses_total_column_for_empty_sex_dimension() {
        let input = "Germany, Exposure table\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Age Female Male Total\n1990 0 10.00 11.00 10.50\n";
        let table = Table::<Year, Age, Empty, ExposureToRisk>::load(input.as_bytes()).unwrap();

        assert_eq!(
            table
                .query(Year(1990), Age::try_from(0).unwrap(), ())
                .map(ToString::to_string),
            Some("10.50".to_owned())
        );
    }

    #[test]
    fn rejects_duplicate_entries() {
        let input = "Germany, Births\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Female Male Total\n1990 10 11 21\n1990 12 13 25\n";
        let result = Table::<Year, Empty, Sex, Births>::load(input.as_bytes());

        assert!(matches!(result, Err(ImportError::DuplicateEntry)));
    }

    #[test]
    fn rejects_missing_required_column() {
        let input = "Germany, Births\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Male Total\n1990 11 21\n";
        let result = Table::<Year, Empty, Sex, usize>::load(input.as_bytes());

        assert!(matches!(result, Err(ImportError::MissingColumn)));
    }

    #[test]
    fn rejects_invalid_age_token() {
        let input = "Germany, Death rates\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Age Female Male Total\n1990 130 1.0 2.0 1.5\n";
        let result = Table::<Year, Age, Sex, f64>::load(input.as_bytes());

        assert!(matches!(result, Err(ImportError::InvalidAge)));
    }
}
