pub use super::{
    Births, CentralDeathRates, Country, Deaths, Empty, LifeExpectanciesAtBirth, LifeTable,
};

use crate::covariates::Sex;
use crate::table::{ImportError, Index, TableIndex};
use crate::values::LifeTableRow;

const BASE_URL: &str = "https://www.mortality.org";
const LOGIN_ENDPOINT: &str = "/Account/Login";
const RETURN_URL: &str = "https://www.mortality.org/Home/Index";
const FORM_URLENCODED: &str = "application/x-www-form-urlencoded";

/// Authenticated session for downloading protected HMD tables.
#[derive(Debug)]
pub struct Session {
    client: reqwest::blocking::Client,
}

impl Session {
    /// Create a new session with the specified username and password.
    pub fn login(username: String, password: String) -> Result<Self, Error> {
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .build()
            .map_err(Error::RequestError)?;

        let login_url = format!("{BASE_URL}{LOGIN_ENDPOINT}");
        let response = client.get(&login_url).send().map_err(Error::RequestError)?;
        let page = response.text().map_err(Error::RequestError)?;
        let token = parse_verification_token(&page).ok_or(Error::InvalidCredentials)?;

        let form = [
            ("ReturnUrl", RETURN_URL),
            ("Email", username.as_str()),
            ("Password", password.as_str()),
            ("__RequestVerificationToken", token.as_str()),
        ];
        let encoded_form = serde_urlencoded::to_string(form).map_err(Error::EncodingError)?;

        let response = client
            .post(&login_url)
            .header(reqwest::header::CONTENT_TYPE, FORM_URLENCODED)
            .body(encoded_form)
            .send()
            .map_err(Error::RequestError)?;

        if response.url().path().contains(LOGIN_ENDPOINT) {
            return Err(Error::InvalidCredentials);
        }

        Ok(Self { client })
    }

    /// Download the specified table for the specified country.
    pub fn download<T: DownloadableTable>(&self, country: Country) -> Result<T, Error> {
        T::download(self, country)
    }

    fn download_file(&self, country: Country, file_name: &str) -> Result<Vec<u8>, Error> {
        let url = format!(
            "{BASE_URL}/File/GetDocument/hmd.v6/{}/STATS/{file_name}.txt",
            country.code()
        );
        let response = self.client.get(url).send().map_err(Error::RequestError)?;
        let response = response.error_for_status().map_err(Error::RequestError)?;
        response
            .bytes()
            .map(|bytes| bytes.to_vec())
            .map_err(Error::RequestError)
    }
}

/// A table that can be downloaded from the Human Mortality Database.
pub trait DownloadableTable: Sized {
    /// Get the file name for the table.
    fn file_name() -> String;

    /// Download the data for the specified country and table.
    fn download(session: &Session, country: Country) -> Result<Self, Error>;
}

impl<Y> DownloadableTable for Births<Y>
where
    Y: Index + TableIndex,
{
    fn file_name() -> String {
        "Births".to_owned()
    }

    fn download(session: &Session, country: Country) -> Result<Self, Error> {
        let file_name = Self::file_name();
        let content = session.download_file(country, &file_name)?;
        Self::load(content.as_slice()).map_err(Error::ImportError)
    }
}

impl<Y, A> DownloadableTable for Deaths<Y, A>
where
    Y: Index + TableIndex,
    A: Index + TableIndex,
{
    fn file_name() -> String {
        format!("Deaths_{}x{}", A::ELEMENTS, Y::ELEMENTS)
    }

    fn download(session: &Session, country: Country) -> Result<Self, Error> {
        let file_name = Self::file_name();
        let content = session.download_file(country, &file_name)?;
        Self::load(content.as_slice()).map_err(Error::ImportError)
    }
}

/// The HMD publishes the combined-sexes period life table as `bltper`, matching how
/// `LifeTable<Y, A>` (`S` defaulting to [`Empty`]) has no sex covariate.
impl<Y, A> DownloadableTable for LifeTable<Y, A, Empty>
where
    Y: Index + TableIndex,
    A: Index + TableIndex,
{
    fn file_name() -> String {
        format!("bltper_{}x{}", A::ELEMENTS, Y::ELEMENTS)
    }

    fn download(session: &Session, country: Country) -> Result<Self, Error> {
        let file_name = Self::file_name();
        let content = session.download_file(country, &file_name)?;
        Self::load(content.as_slice()).map_err(Error::ImportError)
    }
}

#[allow(private_bounds)]
impl<Y, A> LifeTable<Y, A, Sex>
where
    Y: Index + TableIndex,
    A: Index + TableIndex,
{
    /// Download the period life table for the given country, indexed by sex.
    ///
    /// The HMD publishes `fltper` (female) and `mltper` (male) as separate files rather than as
    /// columns within one file, so this fetches both and merges them into a single table. A row
    /// missing from one sex's file (but present in the other) is treated the same as an
    /// explicitly undefined (".") cell: `None`.
    pub fn download(session: &Session, country: Country) -> Result<Self, Error> {
        let female_file = format!("fltper_{}x{}", A::ELEMENTS, Y::ELEMENTS);
        let male_file = format!("mltper_{}x{}", A::ELEMENTS, Y::ELEMENTS);

        let female_content = session.download_file(country, &female_file)?;
        let male_content = session.download_file(country, &male_file)?;

        let (country, female_modified, female_rows) =
            crate::table::parse_rows::<Y, A, Empty, Option<LifeTableRow>, _>(
                female_content.as_slice(),
            )
            .map_err(Error::ImportError)?;
        let (_, male_modified, male_rows) =
            crate::table::parse_rows::<Y, A, Empty, Option<LifeTableRow>, _>(
                male_content.as_slice(),
            )
            .map_err(Error::ImportError)?;

        let merged_rows = merge_life_table_sexes(female_rows, male_rows);
        let last_modified = female_modified.max(male_modified);

        crate::table::build_table(country, last_modified, merged_rows).map_err(Error::ImportError)
    }
}

/// Merges the per-age rows of a female-only and a male-only period life table (each already
/// indexed by [`Empty`]) into rows indexed by [`Sex`]. A row present for only one sex is paired
/// with `None` for the other, consistent with how the HMD's own "." placeholder is treated.
fn merge_life_table_sexes<Y, A>(
    mut female: crate::table::GroupedRows<Y, A, Empty, Option<LifeTableRow>>,
    mut male: crate::table::GroupedRows<Y, A, Empty, Option<LifeTableRow>>,
) -> crate::table::GroupedRows<Y, A, Sex, Option<LifeTableRow>>
where
    Y: Ord + Copy,
    A: Ord + Copy,
{
    let years: std::collections::BTreeSet<Y> =
        female.keys().copied().chain(male.keys().copied()).collect();

    years
        .into_iter()
        .map(|year| {
            let mut female_ages = female.remove(&year).unwrap_or_default();
            let mut male_ages = male.remove(&year).unwrap_or_default();
            let ages: std::collections::BTreeSet<A> = female_ages
                .keys()
                .copied()
                .chain(male_ages.keys().copied())
                .collect();

            let age_map = ages
                .into_iter()
                .map(|age| {
                    let female_value = female_ages.remove(&age).unwrap_or_default();
                    let male_value = male_ages.remove(&age).unwrap_or_default();
                    (age, [(Sex::Female, female_value), (Sex::Male, male_value)])
                })
                .collect();

            (year, age_map)
        })
        .collect()
}

impl<Y> DownloadableTable for LifeExpectanciesAtBirth<Y>
where
    Y: Index + TableIndex,
{
    fn file_name() -> String {
        if Y::ELEMENTS == 1 {
            "E0per".to_owned()
        } else {
            format!("E0per_1x{}", Y::ELEMENTS)
        }
    }

    fn download(session: &Session, country: Country) -> Result<Self, Error> {
        let file_name = Self::file_name();
        let content = session.download_file(country, &file_name)?;
        Self::load(content.as_slice()).map_err(Error::ImportError)
    }
}

impl<Y, A> DownloadableTable for CentralDeathRates<Y, A>
where
    Y: Index + TableIndex,
    A: Index + TableIndex,
{
    fn file_name() -> String {
        format!("Mx_{}x{}", A::ELEMENTS, Y::ELEMENTS)
    }

    fn download(session: &Session, country: Country) -> Result<Self, Error> {
        let file_name = Self::file_name();
        let content = session.download_file(country, &file_name)?;
        Self::load(content.as_slice()).map_err(Error::ImportError)
    }
}

#[derive(Debug)]
/// Errors that can occur during authenticated HMD download and import.
pub enum Error {
    /// The provided credentials are invalid.
    InvalidCredentials,
    /// An error occurred while making the HTTP request.
    RequestError(reqwest::Error),
    /// The downloaded table could not be parsed.
    ImportError(ImportError),
    /// The login form payload could not be encoded.
    EncodingError(serde_urlencoded::ser::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidCredentials => write!(f, "invalid credentials"),
            Error::RequestError(error) => write!(f, "request failed: {error}"),
            Error::ImportError(error) => write!(f, "table import failed: {error}"),
            Error::EncodingError(error) => write!(f, "login payload encoding failed: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InvalidCredentials => None,
            Error::RequestError(error) => Some(error),
            Error::ImportError(error) => Some(error),
            Error::EncodingError(error) => Some(error),
        }
    }
}

fn parse_verification_token(html: &str) -> Option<String> {
    let marker = "name=\"__RequestVerificationToken\"";
    let marker_index = html.find(marker)?;
    let window = &html[marker_index..];

    if let Some(value_marker_index) = window.find("value=\"") {
        let value = &window[(value_marker_index + "value=\"".len())..];
        let end = value.find('"')?;
        return Some(value[..end].to_owned());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::covariates::{Age, Sex, Year};
    use crate::table::Index;
    use crate::{Range, Single};

    #[test]
    fn index_reports_element_count() {
        assert_eq!(Single::<Year>::ELEMENTS, 1);
        assert_eq!(Range::<Year, 5>::ELEMENTS, 5);
        assert_eq!(Single::<Age>::ELEMENTS, 1);
    }

    #[test]
    fn births_download_base_name() {
        assert_eq!(Births::<Single<Year>>::file_name(), "Births");
    }

    #[test]
    fn deaths_download_name_uses_index_size() {
        let file_name = Deaths::<Range<Year, 5>, Single<Age>>::file_name();
        assert_eq!(file_name, "Deaths_1x5");
    }

    #[test]
    fn life_table_download_name_uses_index_size() {
        let file_name = LifeTable::<Range<Year, 10>, Single<Age>, Empty>::file_name();
        assert_eq!(file_name, "bltper_1x10");
    }

    #[test]
    fn life_expectancy_download_name_uses_index_size() {
        let file_name = LifeExpectanciesAtBirth::<Range<Year, 10>>::file_name();
        assert_eq!(file_name, "E0per_1x10");
    }

    #[test]
    fn life_expectancy_download_name_uses_base_file_for_1x1() {
        let file_name = LifeExpectanciesAtBirth::<Single<Year>>::file_name();
        assert_eq!(file_name, "E0per");
    }

    #[test]
    fn central_death_rate_download_name_uses_index_size() {
        let file_name = CentralDeathRates::<Single<Year>, Single<Age>>::file_name();
        assert_eq!(file_name, "Mx_1x1");
    }

    #[test]
    fn downloads_births_table() {
        let session = login_from_env();
        let table = session
            .download::<Births<Single<Year>>>(Country::Germany)
            .expect("failed to download births table");
        assert_eq!(table.country, Country::Germany);
    }

    #[test]
    fn downloads_deaths_table() {
        let session = login_from_env();
        let table = session
            .download::<Deaths<Single<Year>, Single<Age>>>(Country::Germany)
            .expect("failed to download deaths table");
        assert_eq!(table.country, Country::Germany);
    }

    #[test]
    fn downloads_life_table() {
        let session = login_from_env();

        let total = session
            .download::<LifeTable<Single<Year>, Single<Age>>>(Country::Germany)
            .expect("failed to download total life table");
        assert_eq!(total.country, Country::Germany);

        let by_sex =
            LifeTable::<Single<Year>, Single<Age>, Sex>::download(&session, Country::Germany)
                .expect("failed to download sex-indexed life table");
        assert_eq!(by_sex.country, Country::Germany);

        let age0 = Age::try_from(0).unwrap();
        let female_ex = by_sex
            .query(Year(2020), age0, Sex::Female)
            .copied()
            .flatten()
            .map(|row| f64::from(row.ex));
        let male_ex = by_sex
            .query(Year(2020), age0, Sex::Male)
            .copied()
            .flatten()
            .map(|row| f64::from(row.ex));
        assert!(female_ex.is_some() && male_ex.is_some());
        assert_ne!(
            female_ex, male_ex,
            "female and male life tables must differ"
        );
    }

    #[test]
    fn downloads_life_expectancies_table() {
        let session = login_from_env();
        let table = session
            .download::<LifeExpectanciesAtBirth<Single<Year>>>(Country::Germany)
            .expect("failed to download life expectancies table");
        assert_eq!(table.country, Country::Germany);
    }

    #[test]
    fn downloads_central_death_rates_table() {
        let session = login_from_env();
        let attempts = [
            session
                .download::<CentralDeathRates<Single<Year>, Single<Age>>>(Country::Germany)
                .map(|table| table.country),
            session
                .download::<CentralDeathRates<Range<Year, 5>, Single<Age>>>(Country::Germany)
                .map(|table| table.country),
            session
                .download::<CentralDeathRates<Range<Year, 10>, Single<Age>>>(Country::Germany)
                .map(|table| table.country),
            session
                .download::<CentralDeathRates<Single<Year>, Range<Age, 5>>>(Country::Germany)
                .map(|table| table.country),
        ];

        let success = attempts.into_iter().flatten().next();
        assert_eq!(
            success,
            Some(Country::Germany),
            "failed to download central death rates table for tested index combinations"
        );
    }

    fn login_from_env() -> Session {
        let username = std::env::var("HMD_USERNAME").expect("HMD_USERNAME not set");
        let password = std::env::var("HMD_PASSWORD").expect("HMD_PASSWORD not set");
        Session::login(username, password).expect("failed to login to HMD")
    }
}
