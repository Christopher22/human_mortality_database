//! Covariates for querying the Human Mortality Database.

/// A type that can be used as a covariate value in table indices.
pub trait Covariate: Copy + Ord {}

/// Age in years between 0 and 120+.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Age(u8);

impl Age {
    /// The minimum age.
    pub const MIN: Self = Age(0);
    /// The maximum age.
    pub const MAX: Self = Age(120);
}

impl std::fmt::Display for Age {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Age> for u8 {
    fn from(value: Age) -> Self {
        value.0
    }
}

impl TryFrom<u8> for Age {
    type Error = InvalidAgeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= Self::MAX.0 {
            Ok(Self(value))
        } else {
            Err(InvalidAgeError)
        }
    }
}

/// An age out of the valid range (0-120+).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidAgeError;

impl std::fmt::Display for InvalidAgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Age must be between {} and {}.", Age::MIN.0, Age::MAX.0)
    }
}

impl std::error::Error for InvalidAgeError {}

/// A year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Year(pub u16);

impl std::fmt::Display for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Year> for u16 {
    fn from(value: Year) -> Self {
        value.0
    }
}

impl From<Year> for u32 {
    fn from(value: Year) -> Self {
        value.0 as u32
    }
}

impl TryFrom<u32> for Year {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value <= u16::MAX as u32 {
            Ok(Self(value as u16))
        } else {
            Err("Year must be between 0 and 65535")
        }
    }
}

/// The biological sex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sex {
    /// Male
    Male,
    /// Female
    Female,
}

impl std::fmt::Display for Sex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sex::Male => write!(f, "Male"),
            Sex::Female => write!(f, "Female"),
        }
    }
}

impl std::str::FromStr for Sex {
    type Err = ParseSexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim();
        if normalized.eq_ignore_ascii_case("male") || normalized.eq_ignore_ascii_case("m") {
            Ok(Self::Male)
        } else if normalized.eq_ignore_ascii_case("female") || normalized.eq_ignore_ascii_case("f")
        {
            Ok(Self::Female)
        } else {
            Err(ParseSexError)
        }
    }
}

/// A sex token could not be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseSexError;

impl std::fmt::Display for ParseSexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sex must be one of: male, female, m, f")
    }
}

impl std::error::Error for ParseSexError {}

impl Covariate for Age {}
impl Covariate for Year {}
impl Covariate for Sex {}
impl Covariate for () {}
