//! Values available in the Human Mortality Database, such as population size, exposure to risk, and life expectancy at birth.

/// Parsing error for scalar/value cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueError {
    /// The source token cannot be parsed as a number.
    InvalidNumber,
    /// The parsed number violates semantic constraints.
    InvalidValue,
}

/// A value that can be parsed from a table cell.
pub trait Value: Sized {
    /// Parse a value from a single cell token.
    fn parse_value(token: &str) -> Result<Self, ValueError>;
}

/// The size of the population.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PopulationSize(usize);

impl From<usize> for PopulationSize {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for PopulationSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! non_negative_f64_value {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name(f64);

        impl Eq for $name {}

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0.partial_cmp(&other.0).unwrap()
            }
        }

        impl TryFrom<f64> for $name {
            type Error = InvalidValueError;

            fn try_from(value: f64) -> Result<Self, Self::Error> {
                InvalidValueError::check(value).map(Self)
            }
        }

        impl From<$name> for f64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:.2}", self.0)
            }
        }

        impl Value for $name {
            fn parse_value(token: &str) -> Result<Self, ValueError> {
                let value = token
                    .parse::<f64>()
                    .map_err(|_| ValueError::InvalidNumber)?;
                Self::try_from(value).map_err(|_| ValueError::InvalidValue)
            }
        }
    };
}

non_negative_f64_value!(
    Births,
    "The number of births, which is never NaN or negative."
);
non_negative_f64_value!(
    Deaths,
    "The number of deaths, which is never NaN or negative."
);

non_negative_f64_value!(
    ExposureToRisk,
    "The exposure to risk, which is never NaN or negative."
);

non_negative_f64_value!(
    LifeExpectancyAtBirth,
    "The life expectancy at birth, which is never NaN or negative."
);

non_negative_f64_value!(
    CentralDeathRate,
    "Central death rate, which is never NaN or negative."
);
non_negative_f64_value!(
    ProbabilityOfDying,
    "Probability of dying between ages x and x+1, which is never NaN or negative."
);
non_negative_f64_value!(
    AveragePersonYearsLived,
    "Average person-years lived in interval by those who die, which is never NaN or negative."
);
non_negative_f64_value!(
    NumberAlive,
    "Number alive at exact age x, which is never NaN or negative."
);
non_negative_f64_value!(
    NumberDying,
    "Number dying between ages x and x+1, which is never NaN or negative."
);
non_negative_f64_value!(
    PersonYearsLived,
    "Person-years lived between ages x and x+1, which is never NaN or negative."
);
non_negative_f64_value!(
    TotalPersonYearsLived,
    "Total person-years lived above age x, which is never NaN or negative."
);
non_negative_f64_value!(
    RemainingLifeExpectancy,
    "Remaining life expectancy at exact age x, which is never NaN or negative."
);

/// A single row of a life table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LifeTableRow {
    /// Central death rate.
    pub mx: CentralDeathRate,
    /// Probability of dying between ages x and x+1.
    pub qx: ProbabilityOfDying,
    /// Average person-years lived in interval by those who die.
    pub ax: AveragePersonYearsLived,
    /// Number alive at exact age x.
    pub lx: NumberAlive,
    /// Number dying between ages x and x+1.
    pub dx: NumberDying,
    /// Person-years lived between ages x and x+1.
    pub lx_person_years: PersonYearsLived,
    /// Total person-years lived above age x.
    pub tx: TotalPersonYearsLived,
    /// Remaining life expectancy at exact age x.
    pub ex: RemainingLifeExpectancy,
}

impl Eq for LifeTableRow {}

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

impl Value for f64 {
    fn parse_value(token: &str) -> Result<Self, ValueError> {
        token.parse::<f64>().map_err(|_| ValueError::InvalidNumber)
    }
}

impl Value for usize {
    fn parse_value(token: &str) -> Result<Self, ValueError> {
        token
            .parse::<usize>()
            .map_err(|_| ValueError::InvalidNumber)
    }
}

impl Value for PopulationSize {
    fn parse_value(token: &str) -> Result<Self, ValueError> {
        token
            .parse::<usize>()
            .map(PopulationSize::from)
            .map_err(|_| ValueError::InvalidNumber)
    }
}
