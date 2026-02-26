//! Values available in the Human Mortality Database, such as population size, exposure to risk, and life expectancy at birth.

/// The size of the population.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PopulationSize(usize);

impl std::fmt::Display for PopulationSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The exposure to risk, which is never NaN or negative.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ExposureToRisk(f64);

impl Eq for ExposureToRisk {}
impl Ord for ExposureToRisk {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl TryFrom<f64> for ExposureToRisk {
    type Error = InvalidValueError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        InvalidValueError::check(value).map(ExposureToRisk)
    }
}

impl std::fmt::Display for ExposureToRisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

/// The life expectancy at birth, which is never NaN or negative.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LifeExpectancyAtBirth(f64);
impl Eq for LifeExpectancyAtBirth {}
impl Ord for LifeExpectancyAtBirth {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl TryFrom<f64> for LifeExpectancyAtBirth {
    type Error = InvalidValueError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        InvalidValueError::check(value).map(LifeExpectancyAtBirth)
    }
}

impl std::fmt::Display for LifeExpectancyAtBirth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

/// An error indicating that a value is invalid (e.g., NaN or negative).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvalidValueError {
    /// The value is NaN.
    IsNaN,
    /// The value is negative.
    IsNegative,
}

impl InvalidValueError {
    fn check(value: f64) -> Result<f64, Self> {
        if value.is_nan() {
            Err(Self::IsNaN)
        } else if value < 0.0 {
            Err(Self::IsNegative)
        } else {
            Ok(value)
        }
    }
}

impl std::fmt::Display for InvalidValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidValueError::IsNaN => write!(f, "Value must not be NaN."),
            InvalidValueError::IsNegative => write!(f, "Value must not be negative."),
        }
    }
}

impl std::error::Error for InvalidValueError {}
