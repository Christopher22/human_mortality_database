use crate::covariates::{Age, Sex, Year};

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
        let [(s1, v1), (_, v2)] = values;
        match value {
            s1 => Some(v1),
            _ => Some(v2),
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
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.start.cmp(&other.start))
    }
}

impl<T: Index, const N: usize> Ord for Range<T, N> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
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
                    std::cmp::Ordering::Equal
                } else if value < range.start {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
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
    pub last_modified: chrono::NaiveDate,
    data: Y::Container<A::Container<S::Container<D>>>,
}

impl<Y: Index, A: Index, S: Index, D> Table<Y, A, S, D> {
    /// Querys the table for the given year and age, returning the associated data if found.
    pub fn query(&self, year: Y::Value, age: A::Value, sex: S::Value) -> Option<&D> {
        Y::find(&self.data, year)
            .and_then(|ages| A::find(ages, age))
            .and_then(|sexes| S::find(sexes, sex))
    }
}
