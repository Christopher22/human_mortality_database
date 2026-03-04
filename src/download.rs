pub use super::{Births, CentralDeathRates, Country, Deaths, LifeExpectanciesAtBirth, LifeTable};

use crate::table::{ImportError, Index, TableIndex};

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

impl<Y, A> DownloadableTable for LifeTable<Y, A>
where
    Y: Index + TableIndex,
    A: Index + TableIndex,
{
    fn file_name() -> String {
        format!("fltper_{}x{}", A::ELEMENTS, Y::ELEMENTS)
    }

    fn download(session: &Session, country: Country) -> Result<Self, Error> {
        let file_name = Self::file_name();
        let content = session.download_file(country, &file_name)?;
        Self::load(content.as_slice()).map_err(Error::ImportError)
    }
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
    use crate::covariates::{Age, Year};
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
        let file_name = LifeTable::<Range<Year, 10>, Single<Age>>::file_name();
        assert_eq!(file_name, "fltper_1x10");
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
        let table = session
            .download::<LifeTable<Single<Year>, Single<Age>>>(Country::Germany)
            .expect("failed to download life table");
        assert_eq!(table.country, Country::Germany);
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
