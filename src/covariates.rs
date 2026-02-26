//! Covariates for querying the Human Mortality Database.

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
