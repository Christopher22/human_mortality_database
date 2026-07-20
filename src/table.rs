use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::io::BufRead;

use chrono::NaiveDate;

use crate::covariates::{Age, Covariate, Sex, Year};
use crate::values::{LifeTableRow, Value, ValueError};

/// A trait for types that can be used as indices in the table.
pub trait Index: Copy + Ord {
    /// The type of the index value.
    type Value: Covariate;
    /// The type of the container that holds the values associated with the index.
    type Container<T>;
    /// The number of elements represented by one index key.
    const ELEMENTS: usize;

    /// Finds the value associated with the given index value.
    /// The list of values must be sorted by the index value and its elements should be unique.
    fn find<T>(values: &Self::Container<T>, value: Self::Value) -> Option<&T>;
}

/// A trait for indices that actually contain values.
pub trait NonEmptyIndex: Index {}

/// A single indexed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Single<T: Covariate>(pub T);

impl<T: Covariate> Index for Single<T> {
    type Value = T;
    type Container<U> = Vec<(Self, U)>;
    const ELEMENTS: usize = 1;

    fn find<U>(values: &Self::Container<U>, value: T) -> Option<&U> {
        values
            .binary_search_by_key(&value, |(index, _)| index.0)
            .ok()
            .map(|index| &values[index].1)
    }
}

impl<T: Covariate> NonEmptyIndex for Single<T> {}

impl Index for Sex {
    type Value = Self;
    type Container<T> = [(Self, T); 2];
    const ELEMENTS: usize = 2;

    fn find<T>(values: &Self::Container<T>, value: Self) -> Option<&T> {
        match value {
            Sex::Female => Some(&values[0].1),
            Sex::Male => Some(&values[1].1),
        }
    }
}

impl NonEmptyIndex for Sex {}

/// A range of values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range<T: Covariate, const N: usize> {
    start: T,
    end: T,
}

impl<T: Covariate, const N: usize> Range<T, N> {
    /// Check if the range contains the given value.
    pub fn contains(&self, value: T) -> bool {
        self.start <= value && value <= self.end
    }
}

impl<T: Covariate, const N: usize> PartialOrd for Range<T, N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Covariate, const N: usize> Ord for Range<T, N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl<T: Covariate, const N: usize> Index for Range<T, N> {
    type Value = T;
    type Container<I> = Vec<(Self, I)>;
    const ELEMENTS: usize = N;

    fn find<U>(values: &Self::Container<U>, value: T) -> Option<&U> {
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

impl<T: Covariate, const N: usize> NonEmptyIndex for Range<T, N> {}

/// An empty index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Empty;
impl Index for Empty {
    type Value = ();
    type Container<T> = T;
    const ELEMENTS: usize = 1;

    fn find<T>(values: &T, _value: Self::Value) -> Option<&T> {
        Some(values)
    }
}

/// A country for which data is available in the HMD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Country {
    /// Australia
    Australia,
    /// Austria
    Austria,
    /// Belarus
    Belarus,
    /// Belgium
    Belgium,
    /// Bulgaria
    Bulgaria,
    /// Canada
    Canada,
    /// Chile
    Chile,
    /// Croatia
    Croatia,
    /// Czechia
    Czechia,
    /// Denmark
    Denmark,
    /// Estonia
    Estonia,
    /// Finland
    Finland,
    /// France (total population)
    FranceTotalPopulation,
    /// France (civilian population)
    FranceCivilianPopulation,
    /// Germany
    Germany,
    /// East Germany
    EastGermany,
    /// West Germany
    WestGermany,
    /// Greece
    Greece,
    /// Hong Kong
    HongKong,
    /// Hungary
    Hungary,
    /// Iceland
    Iceland,
    /// Ireland
    Ireland,
    /// Israel
    Israel,
    /// Italy
    Italy,
    /// Japan
    Japan,
    /// Latvia
    Latvia,
    /// Lithuania
    Lithuania,
    /// Luxembourg
    Luxembourg,
    /// Netherlands
    Netherlands,
    /// New Zealand (total population)
    NewZealandTotalPopulation,
    /// New Zealand (Maori)
    NewZealandMaori,
    /// New Zealand (Non-Maori)
    NewZealandNonMaori,
    /// Norway
    Norway,
    /// Poland
    Poland,
    /// Portugal
    Portugal,
    /// Republic of Korea
    RepublicOfKorea,
    /// Russia
    Russia,
    /// Slovakia
    Slovakia,
    /// Slovenia
    Slovenia,
    /// Spain
    Spain,
    /// Sweden
    Sweden,
    /// Switzerland
    Switzerland,
    /// Taiwan
    Taiwan,
    /// United Kingdom (total population)
    UnitedKingdomTotalPopulation,
    /// England & Wales (total population)
    EnglandAndWalesTotalPopulation,
    /// England & Wales (civilian population)
    EnglandAndWalesCivilianPopulation,
    /// Scotland
    Scotland,
    /// Northern Ireland
    NorthernIreland,
    /// U.S.A.
    UnitedStatesOfAmerica,
    /// Ukraine
    Ukraine,
}

impl Country {
    /// Returns the HMD code for the country.
    pub fn code(&self) -> &'static str {
        match self {
            Country::Australia => "AUS",
            Country::Austria => "AUT",
            Country::Belarus => "BLR",
            Country::Belgium => "BEL",
            Country::Bulgaria => "BGR",
            Country::Canada => "CAN",
            Country::Chile => "CHL",
            Country::Croatia => "HRV",
            Country::Czechia => "CZE",
            Country::Denmark => "DNK",
            Country::Estonia => "EST",
            Country::Finland => "FIN",
            Country::FranceTotalPopulation => "FRATNP",
            Country::FranceCivilianPopulation => "FRACNP",
            Country::Germany => "DEUTNP",
            Country::EastGermany => "DEUTE",
            Country::WestGermany => "DEUTW",
            Country::Greece => "GRC",
            Country::HongKong => "HKG",
            Country::Hungary => "HUN",
            Country::Iceland => "ISL",
            Country::Ireland => "IRL",
            Country::Israel => "ISR",
            Country::Italy => "ITA",
            Country::Japan => "JPN",
            Country::Latvia => "LVA",
            Country::Lithuania => "LTU",
            Country::Luxembourg => "LUX",
            Country::Netherlands => "NLD",
            Country::NewZealandTotalPopulation => "NZL_NP",
            Country::NewZealandMaori => "NZL_MA",
            Country::NewZealandNonMaori => "NZL_NM",
            Country::Norway => "NOR",
            Country::Poland => "POL",
            Country::Portugal => "PRT",
            Country::RepublicOfKorea => "KOR",
            Country::Russia => "RUS",
            Country::Slovakia => "SVK",
            Country::Slovenia => "SVN",
            Country::Spain => "ESP",
            Country::Sweden => "SWE",
            Country::Switzerland => "CHE",
            Country::Taiwan => "TWN",
            Country::UnitedKingdomTotalPopulation => "GBR_NP",
            Country::EnglandAndWalesTotalPopulation => "GBRTENW",
            Country::EnglandAndWalesCivilianPopulation => "GBRCENW",
            Country::Scotland => "GBR_SCO",
            Country::NorthernIreland => "GBR_NIR",
            Country::UnitedStatesOfAmerica => "USA",
            Country::Ukraine => "UKR",
        }
    }

    /// Parse a country from its HMD code.
    pub fn from_code(code: &str) -> Option<Self> {
        match code.trim().to_ascii_uppercase().as_str() {
            "AUS" => Some(Country::Australia),
            "AUT" => Some(Country::Austria),
            "BLR" => Some(Country::Belarus),
            "BEL" => Some(Country::Belgium),
            "BGR" => Some(Country::Bulgaria),
            "CAN" => Some(Country::Canada),
            "CHL" => Some(Country::Chile),
            "HRV" => Some(Country::Croatia),
            "CZE" => Some(Country::Czechia),
            "DNK" => Some(Country::Denmark),
            "EST" => Some(Country::Estonia),
            "FIN" => Some(Country::Finland),
            "FRATNP" => Some(Country::FranceTotalPopulation),
            "FRACNP" => Some(Country::FranceCivilianPopulation),
            "DEUTNP" => Some(Country::Germany),
            "DEUTE" => Some(Country::EastGermany),
            "DEUTW" => Some(Country::WestGermany),
            "GRC" => Some(Country::Greece),
            "HKG" => Some(Country::HongKong),
            "HUN" => Some(Country::Hungary),
            "ISL" => Some(Country::Iceland),
            "IRL" => Some(Country::Ireland),
            "ISR" => Some(Country::Israel),
            "ITA" => Some(Country::Italy),
            "JPN" => Some(Country::Japan),
            "LVA" => Some(Country::Latvia),
            "LTU" => Some(Country::Lithuania),
            "LUX" => Some(Country::Luxembourg),
            "NLD" => Some(Country::Netherlands),
            "NZL_NP" => Some(Country::NewZealandTotalPopulation),
            "NZL_MA" => Some(Country::NewZealandMaori),
            "NZL_NM" => Some(Country::NewZealandNonMaori),
            "NOR" => Some(Country::Norway),
            "POL" => Some(Country::Poland),
            "PRT" => Some(Country::Portugal),
            "KOR" => Some(Country::RepublicOfKorea),
            "RUS" => Some(Country::Russia),
            "SVK" => Some(Country::Slovakia),
            "SVN" => Some(Country::Slovenia),
            "ESP" => Some(Country::Spain),
            "SWE" => Some(Country::Sweden),
            "CHE" => Some(Country::Switzerland),
            "TWN" => Some(Country::Taiwan),
            "GBR_NP" => Some(Country::UnitedKingdomTotalPopulation),
            "GBRTENW" => Some(Country::EnglandAndWalesTotalPopulation),
            "GBRCENW" => Some(Country::EnglandAndWalesCivilianPopulation),
            "GBR_SCO" => Some(Country::Scotland),
            "GBR_NIR" => Some(Country::NorthernIreland),
            "USA" => Some(Country::UnitedStatesOfAmerica),
            "UKR" => Some(Country::Ukraine),
            _ => None,
        }
    }

    /// Finds the country whose display name the given metadata line starts with.
    ///
    /// The country name cannot reliably be isolated by splitting on the first comma: some HMD
    /// display names contain an embedded comma of their own (e.g. "England and Wales, Total
    /// Population"). Matching the longest known display name that prefixes the line handles both
    /// plain names and comma-bearing ones uniformly, and resolves the ambiguity between e.g.
    /// "New Zealand" and "New Zealand -- Maori" by preferring the more specific (longer) name.
    fn from_line_prefix(line: &str) -> Option<Self> {
        const NAMES: &[(&str, Country)] = &[
            ("Australia", Country::Australia),
            ("Austria", Country::Austria),
            ("Belarus", Country::Belarus),
            ("Belgium", Country::Belgium),
            ("Bulgaria", Country::Bulgaria),
            ("Canada", Country::Canada),
            ("Chile", Country::Chile),
            ("Croatia", Country::Croatia),
            ("Czechia", Country::Czechia),
            ("Denmark", Country::Denmark),
            ("Estonia", Country::Estonia),
            ("Finland", Country::Finland),
            ("France Total population", Country::FranceTotalPopulation),
            (
                "France Civilian population",
                Country::FranceCivilianPopulation,
            ),
            ("France", Country::FranceTotalPopulation),
            ("East Germany", Country::EastGermany),
            ("West Germany", Country::WestGermany),
            ("Germany", Country::Germany),
            ("Greece", Country::Greece),
            ("Hong Kong", Country::HongKong),
            ("Hungary", Country::Hungary),
            ("Iceland", Country::Iceland),
            ("Ireland", Country::Ireland),
            ("Israel", Country::Israel),
            ("Italy", Country::Italy),
            ("Japan", Country::Japan),
            ("Latvia", Country::Latvia),
            ("Lithuania", Country::Lithuania),
            ("Luxemburg", Country::Luxembourg),
            ("Luxembourg", Country::Luxembourg),
            ("Netherlands", Country::Netherlands),
            ("New Zealand -- Maori", Country::NewZealandMaori),
            ("New Zealand -- Non-Maori", Country::NewZealandNonMaori),
            (
                "New Zealand Total population",
                Country::NewZealandTotalPopulation,
            ),
            ("New Zealand", Country::NewZealandTotalPopulation),
            ("Maori", Country::NewZealandMaori),
            ("Non-Maori", Country::NewZealandNonMaori),
            ("Norway", Country::Norway),
            ("Poland", Country::Poland),
            ("Portugal", Country::Portugal),
            ("Republic of Korea", Country::RepublicOfKorea),
            ("Russia", Country::Russia),
            ("Slovakia", Country::Slovakia),
            ("Slovenia", Country::Slovenia),
            ("Spain", Country::Spain),
            ("Sweden", Country::Sweden),
            ("Switzerland", Country::Switzerland),
            ("Taiwan", Country::Taiwan),
            (
                "United Kingdom Total Population",
                Country::UnitedKingdomTotalPopulation,
            ),
            ("United Kingdom", Country::UnitedKingdomTotalPopulation),
            ("U.K.", Country::UnitedKingdomTotalPopulation),
            (
                "England and Wales, Total Population",
                Country::EnglandAndWalesTotalPopulation,
            ),
            (
                "England and Wales, Civilian National Population",
                Country::EnglandAndWalesCivilianPopulation,
            ),
            (
                "England & Wales Total Population",
                Country::EnglandAndWalesTotalPopulation,
            ),
            (
                "England & Wales Civilian Population",
                Country::EnglandAndWalesCivilianPopulation,
            ),
            ("Scotland", Country::Scotland),
            ("Northern Ireland", Country::NorthernIreland),
            (
                "The United States of America",
                Country::UnitedStatesOfAmerica,
            ),
            ("U.S.A.", Country::UnitedStatesOfAmerica),
            ("USA", Country::UnitedStatesOfAmerica),
            ("Ukraine", Country::Ukraine),
        ];

        let line = line.trim_start();
        NAMES
            .iter()
            .filter(|(name, _)| line.starts_with(name))
            .max_by_key(|(name, _)| name.len())
            .map(|&(_, country)| country)
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
        Y: TableIndex,
        A: TableIndex,
        D: DataParser<S>,
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

impl<Y: NonEmptyIndex + TableIndex, A: NonEmptyIndex + TableIndex, S: Index, D: DataParser<S>>
    std::ops::Index<(Y::Value, A::Value, S::Value)> for Table<Y, A, S, D>
{
    type Output = D;

    fn index(&self, (year, age, sex): (Y::Value, A::Value, S::Value)) -> &Self::Output {
        self.query(year, age, sex).expect("not found")
    }
}

impl<Y: NonEmptyIndex + TableIndex, A: NonEmptyIndex + TableIndex, D: DataParser<Empty>>
    std::ops::Index<(Y::Value, A::Value)> for Table<Y, A, Empty, D>
{
    type Output = D;

    fn index(&self, (year, age): (Y::Value, A::Value)) -> &Self::Output {
        Y::find(&self.data, year)
            .and_then(|ages| A::find(ages, age))
            .expect("not found")
    }
}

impl<Y: NonEmptyIndex + TableIndex, S: NonEmptyIndex, D: DataParser<S>>
    std::ops::Index<(Y::Value, S::Value)> for Table<Y, Empty, S, D>
{
    type Output = D;

    fn index(&self, (year, sex): (Y::Value, S::Value)) -> &Self::Output {
        Y::find(&self.data, year)
            .and_then(|sexes| S::find(sexes, sex))
            .expect("not found")
    }
}

impl<Y: NonEmptyIndex + TableIndex, D: DataParser<Empty>> std::ops::Index<Y::Value>
    for Table<Y, Empty, Empty, D>
{
    type Output = D;

    fn index(&self, year: Y::Value) -> &Self::Output {
        Y::find(&self.data, year).expect("not found")
    }
}

fn load_impl<Y, A, S, D, R>(reader: R) -> Result<Table<Y, A, S, D>, ImportError>
where
    Y: TableIndex,
    A: TableIndex,
    S: Index,
    R: std::io::Read,
    D: DataParser<S>,
{
    let (country, last_modified, grouped) = parse_rows::<Y, A, S, D, R>(reader)?;
    build_table(country, last_modified, grouped)
}

/// Rows of an HMD table file grouped by year and age, prior to conversion into a [`Table`]'s
/// opaque container representation.
pub(crate) type GroupedRows<Y, A, S, D> = BTreeMap<Y, BTreeMap<A, <S as Index>::Container<D>>>;

/// Parses the metadata, header, and data rows of an HMD table file, without converting the
/// resulting rows into a [`Table`]'s opaque container representation.
///
/// This is split out from [`load_impl`] so that callers who need to combine rows from more than
/// one source file before building a [`Table`] (e.g. merging a country's separate `fltper` and
/// `mltper` period life table files into one table indexed by sex) can reuse the row-level
/// parsing logic.
#[allow(private_bounds, clippy::type_complexity)]
pub(crate) fn parse_rows<Y, A, S, D, R>(
    reader: R,
) -> Result<(Country, NaiveDate, GroupedRows<Y, A, S, D>), ImportError>
where
    Y: TableIndex,
    A: TableIndex,
    S: Index,
    R: std::io::Read,
    D: DataParser<S>,
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

    Ok((country, last_modified, grouped))
}

/// Builds a [`Table`] from already-parsed rows, e.g. the output of [`parse_rows`] or rows merged
/// from more than one source file.
pub(crate) fn build_table<Y, A, S, D>(
    country: Country,
    last_modified: NaiveDate,
    grouped: GroupedRows<Y, A, S, D>,
) -> Result<Table<Y, A, S, D>, ImportError>
where
    Y: TableIndex,
    A: TableIndex,
    S: Index,
{
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
    let country = Country::from_line_prefix(line).ok_or(ImportError::UnknownCountry)?;

    let last_modified = line
        .split("Last modified:")
        .nth(1)
        .and_then(|rest| rest.split(';').next())
        .map(str::trim)
        .ok_or(ImportError::MissingLastModified)?;
    let last_modified = parse_last_modified_date(last_modified)?;
    Ok((country, last_modified))
}

/// Parses a "Last modified" date such as "03 Jun 2022" or "09 juin 2026".
///
/// The HMD occasionally serves this field with a French month abbreviation instead of English
/// (observed for the U.S.A. dataset), so the month name is matched case-insensitively against
/// both languages rather than relying on `chrono`'s locale-fixed "%b" parser.
fn parse_last_modified_date(text: &str) -> Result<NaiveDate, ImportError> {
    fn parse_month_name(token: &str) -> Option<u32> {
        let normalized = token.to_ascii_lowercase();
        let normalized = normalized.trim_end_matches('.');
        match normalized {
            "jan" | "janv" | "janvier" => Some(1),
            "feb" | "fev" | "févr" | "fevrier" | "février" => Some(2),
            "mar" | "mars" => Some(3),
            "apr" | "avr" | "avril" => Some(4),
            "may" | "mai" => Some(5),
            "jun" | "juin" => Some(6),
            "jul" | "juil" | "juillet" => Some(7),
            "aug" | "aou" | "aoû" | "aout" | "août" => Some(8),
            "sep" | "sept" | "septembre" => Some(9),
            "oct" | "octobre" => Some(10),
            "nov" | "novembre" => Some(11),
            "dec" | "dece" | "déc" | "decembre" | "décembre" => Some(12),
            _ => None,
        }
    }

    let mut parts = text.split_whitespace();
    let day = parts.next().and_then(|token| token.parse::<u32>().ok());
    let month = parts.next().and_then(parse_month_name);
    let year = parts.next().and_then(|token| token.parse::<i32>().ok());

    match (day, month, year) {
        (Some(day), Some(month), Some(year)) => {
            NaiveDate::from_ymd_opt(year, month, day).ok_or(ImportError::InvalidLastModified)
        }
        _ => Err(ImportError::InvalidLastModified),
    }
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

fn map_value_error(error: ValueError) -> ImportError {
    match error {
        ValueError::InvalidNumber => ImportError::InvalidNumber,
        ValueError::InvalidValue => ImportError::InvalidValue,
    }
}

fn parse_required_value<D: Value>(
    header: &Header,
    fields: &[&str],
    column: &str,
) -> Result<D, ImportError> {
    let index = header.require(column)?;
    D::parse_value(fields[index]).map_err(map_value_error)
}

pub(crate) trait TableIndex: Index {
    fn parse(token: Option<&str>, field: &str) -> Result<Self, ImportError>;
    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError>;
}

impl TableIndex for Single<Year> {
    fn parse(token: Option<&str>, field: &str) -> Result<Self, ImportError> {
        if field != "year" {
            return Err(ImportError::InvalidYear);
        }
        let token = token.ok_or(ImportError::MissingColumn)?;
        token
            .parse::<u16>()
            .map(Year)
            .map(Single)
            .map_err(|_| ImportError::InvalidYear)
    }

    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError> {
        Ok(map.into_iter().collect())
    }
}

impl<const N: usize> TableIndex for Range<Year, N> {
    fn parse(token: Option<&str>, field: &str) -> Result<Self, ImportError> {
        if field != "year" {
            return Err(ImportError::InvalidRange);
        }
        parse_numeric_range(token.ok_or(ImportError::MissingColumn)?).map(|(start, end)| Self {
            start: Year(start),
            end: Year(end),
        })
    }

    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError> {
        Ok(map.into_iter().collect())
    }
}

impl TableIndex for Single<Age> {
    fn parse(token: Option<&str>, field: &str) -> Result<Self, ImportError> {
        if field != "age" {
            return Err(ImportError::InvalidAge);
        }
        parse_age_token(token.ok_or(ImportError::MissingColumn)?).map(Single)
    }

    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError> {
        Ok(map.into_iter().collect())
    }
}

impl<const N: usize> TableIndex for Range<Age, N> {
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

    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError> {
        Ok(map.into_iter().collect())
    }
}

impl TableIndex for Empty {
    fn parse(_token: Option<&str>, _field: &str) -> Result<Self, ImportError> {
        Ok(Empty)
    }

    fn from_btree<T>(map: BTreeMap<Self, T>) -> Result<Self::Container<T>, ImportError> {
        map.into_values().next().ok_or(ImportError::EmptyInput)
    }
}

trait DataParser<S: Index>: Sized {
    fn parse_data(header: &Header, fields: &[&str]) -> Result<S::Container<Self>, ImportError>;
}

impl<D: Value> DataParser<Sex> for D {
    fn parse_data(
        header: &Header,
        fields: &[&str],
    ) -> Result<<Sex as Index>::Container<Self>, ImportError> {
        let female = header.require("female")?;
        let male = header.require("male")?;
        let female = D::parse_value(fields[female]).map_err(map_value_error)?;
        let male = D::parse_value(fields[male]).map_err(map_value_error)?;
        Ok([(Sex::Female, female), (Sex::Male, male)])
    }
}

impl<D: Value> DataParser<Empty> for D {
    fn parse_data(
        header: &Header,
        fields: &[&str],
    ) -> Result<<Empty as Index>::Container<Self>, ImportError> {
        if let Some(total) = header.find("total") {
            return D::parse_value(fields[total]).map_err(map_value_error);
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

        D::parse_value(fields[data_columns[0]]).map_err(map_value_error)
    }
}

impl DataParser<Empty> for LifeTableRow {
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

impl DataParser<Empty> for Option<LifeTableRow> {
    fn parse_data(
        header: &Header,
        fields: &[&str],
    ) -> Result<<Empty as Index>::Container<Self>, ImportError> {
        // The HMD blanks out entire rows (all columns at once) with "." during historical gaps
        // in a country's data (e.g. Belgium during WWI). "mx" is used as a sentinel for the row:
        // if it is undefined the whole row is undefined.
        let mx_index = header.require("mx")?;
        if fields[mx_index] == "." {
            return Ok(None);
        }
        LifeTableRow::parse_data(header, fields).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::{Births, Deaths, ExposureToRisk, LifeExpectancyAtBirth, LifeTableRow};

    #[test]
    fn loads_births_1x1_with_sex_dimension() {
        let input = "Germany,  Births (1-year)\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Female Male Total\n1990 10 11 21\n1991 12 13 25\n";
        let table = Table::<Single<Year>, Empty, Sex, Births>::load(input.as_bytes()).unwrap();

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
        let table = Table::<Single<Year>, Single<Age>, Sex, f64>::load(input.as_bytes()).unwrap();

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
    fn loads_mx_1x1_tolerates_dot_placeholder_for_undefined_rate() {
        use crate::values::CentralDeathRate;

        let input = "Germany, Death rates (period 1x1),\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Age Female Male Total\n1990 0 0.10 0.20 0.15\n1990 110+ . 1.20 1.20\n";
        let table = Table::<Single<Year>, Single<Age>, Sex, Option<CentralDeathRate>>::load(
            input.as_bytes(),
        )
        .unwrap();

        assert_eq!(
            table.query(Year(1990), Age::try_from(110).unwrap(), Sex::Female),
            Some(&None)
        );
        assert_eq!(
            table
                .query(Year(1990), Age::try_from(110).unwrap(), Sex::Male)
                .copied()
                .flatten()
                .map(f64::from),
            Some(1.20)
        );
    }

    #[test]
    fn loads_deaths_1x5_with_year_ranges() {
        let input = "Germany, Deaths (period 1x5),\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Age Female Male Total\n1990-1994 0 100.0 120.0 220.0\n1995-1999 0 110.0 130.0 240.0\n";
        let table =
            Table::<Range<Year, 5>, Single<Age>, Sex, Deaths>::load(input.as_bytes()).unwrap();

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
        let table =
            Table::<Single<Year>, Range<Age, 5>, Sex, Deaths>::load(input.as_bytes()).unwrap();

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
            Table::<Range<Year, 10>, Single<Age>, Empty, LifeTableRow>::load(input.as_bytes())
                .unwrap();

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
        let table = Table::<Single<Year>, Single<Age>, Empty, LifeTableRow>::load(input.as_bytes())
            .unwrap();

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
        let table =
            Table::<Single<Year>, Single<Age>, Empty, ExposureToRisk>::load(input.as_bytes())
                .unwrap();

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
        let result = Table::<Single<Year>, Empty, Sex, Births>::load(input.as_bytes());

        assert!(matches!(result, Err(ImportError::DuplicateEntry)));
    }

    #[test]
    fn rejects_missing_required_column() {
        let input = "Germany, Births\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Male Total\n1990 11 21\n";
        let result = Table::<Single<Year>, Empty, Sex, usize>::load(input.as_bytes());

        assert!(matches!(result, Err(ImportError::MissingColumn)));
    }

    #[test]
    fn rejects_invalid_age_token() {
        let input = "Germany, Death rates\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Age Female Male Total\n1990 130 1.0 2.0 1.5\n";
        let result = Table::<Single<Year>, Single<Age>, Sex, f64>::load(input.as_bytes());

        assert!(matches!(result, Err(ImportError::InvalidAge)));
    }

    #[test]
    fn parses_non_germany_country_metadata() {
        let input = "Australia, Births\tLast modified: 03 Jun 2022;  Methods Protocol: v6 (2017)\n\nYear Female Male Total\n1990 10 11 21\n";
        let table = Table::<Single<Year>, Empty, Sex, Births>::load(input.as_bytes()).unwrap();

        assert_eq!(table.country.code(), "AUS");
        assert_eq!(table.country, Country::Australia);
    }

    #[test]
    fn index_operator_3d_year_age_sex() {
        use crate::values::Births;

        let input = "Germany, Births 1x1    Last modified: 03 Jun 2022\n\nYear Age Female Male Total\n1990 0 10 11 21\n1990 1 12 13 25\n";

        let table =
            Table::<Single<Year>, Single<Age>, Sex, Births>::load(input.as_bytes()).unwrap();

        let age0 = Age::try_from(0).unwrap();
        let age1 = Age::try_from(1).unwrap();

        assert_eq!(f64::from(table[(Year(1990), age0, Sex::Female)]), 10.0);
        assert_eq!(f64::from(table[(Year(1990), age0, Sex::Male)]), 11.0);
        assert_eq!(f64::from(table[(Year(1990), age1, Sex::Female)]), 12.0);
        assert_eq!(f64::from(table[(Year(1990), age1, Sex::Male)]), 13.0);
    }

    #[test]
    #[should_panic(expected = "not found")]
    fn index_operator_3d_panics_if_missing() {
        use crate::values::Births;

        let input = "Germany, Births 1x1    Last modified: 03 Jun 2022\n\nYear Age Female Male Total\n1990 0 10 11 21\n";

        let table =
            Table::<Single<Year>, Single<Age>, Sex, Births>::load(input.as_bytes()).unwrap();

        let bad_age = Age::try_from(5).unwrap();

        let _ = table[(Year(1990), bad_age, Sex::Female)];
    }

    #[test]
    fn index_operator_2d_year_age() {
        use crate::values::LifeExpectancyAtBirth;

        let input = "Germany, Life expectancy   Last modified: 03 Jun 2022\n\nYear Age ex\n1990 0 75.0\n1991 0 76.0\n";

        let table = Table::<Single<Year>, Single<Age>, Empty, LifeExpectancyAtBirth>::load(
            input.as_bytes(),
        )
        .unwrap();

        let a0 = Age::try_from(0).unwrap();
        assert_eq!(f64::from(table[(Year(1990), a0)]), 75.0);
        assert_eq!(f64::from(table[(Year(1991), a0)]), 76.0);
    }

    #[test]
    #[should_panic(expected = "not found")]
    fn index_operator_2d_year_age_panics() {
        use crate::values::LifeExpectancyAtBirth;

        let input =
            "Germany, Life expectancy   Last modified: 03 Jun 2022\n\nYear Age ex\n1990 0 75.0\n";

        let table = Table::<Single<Year>, Single<Age>, Empty, LifeExpectancyAtBirth>::load(
            input.as_bytes(),
        )
        .unwrap();

        let a0 = Age::try_from(0).unwrap();
        let _ = table[(Year(1991), a0)];
    }

    #[test]
    fn index_operator_2d_year_sex() {
        use crate::values::Deaths;

        let input = "Germany, Deaths   Last modified: 03 Jun 2022\n\nYear Female Male Total\n1990 100 120 220\n1991 110 130 240\n";

        let table = Table::<Single<Year>, Empty, Sex, Deaths>::load(input.as_bytes()).unwrap();

        assert_eq!(f64::from(table[(Year(1990), Sex::Female)]), 100.0);
        assert_eq!(f64::from(table[(Year(1990), Sex::Male)]), 120.0);
        assert_eq!(f64::from(table[(Year(1991), Sex::Female)]), 110.0);
    }

    #[test]
    #[should_panic(expected = "not found")]
    fn index_operator_2d_year_sex_panics() {
        use crate::values::Deaths;

        let input = "Germany, Deaths   Last modified: 03 Jun 2022\n\nYear Female Male Total\n1990 100 120 220\n";

        let table = Table::<Single<Year>, Empty, Sex, Deaths>::load(input.as_bytes()).unwrap();

        let _ = table[(Year(1991), Sex::Female)];
    }

    #[test]
    fn index_operator_1d_year_only() {
        use crate::values::ExposureToRisk;

        let input = "Germany, Exposure   Last modified: 03 Jun 2022\n\nYear Total\n1990 100.0\n1991 110.0\n";

        let table =
            Table::<Single<Year>, Empty, Empty, ExposureToRisk>::load(input.as_bytes()).unwrap();

        assert_eq!(f64::from(table[Year(1990)]), 100.0);
        assert_eq!(f64::from(table[Year(1991)]), 110.0);
    }

    #[test]
    #[should_panic(expected = "not found")]
    fn index_operator_1d_year_panics() {
        use crate::values::ExposureToRisk;

        let input = "Germany, Exposure   Last modified: 03 Jun 2022\n\nYear Total\n1990 100.0\n";

        let table =
            Table::<Single<Year>, Empty, Empty, ExposureToRisk>::load(input.as_bytes()).unwrap();

        let _ = table[Year(1991)];
    }
}
