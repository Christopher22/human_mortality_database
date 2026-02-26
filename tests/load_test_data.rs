use human_mortality_database::covariates::{Age, Sex, Year};
use human_mortality_database::{
    Births, CentralDeathRates, Deaths, LifeExpectanciesAtBirth, Range, Single,
};

#[test]
fn loads_births_txt() {
    let table =
        Births::<Single<Year>>::load(include_str!("test_data/Births.txt").as_bytes()).unwrap();

    assert_eq!(
        table
            .query(Year(1990), (), Sex::Female)
            .map(|value| f64::from(*value)),
        Some(440_296.0)
    );
}

#[test]
fn loads_deaths_1x5_txt() {
    let table = Deaths::<Range<Year, 5>, Single<Age>>::load(
        include_str!("test_data/Deaths_1x5.txt").as_bytes(),
    )
    .unwrap();

    assert_eq!(
        table
            .query(Year(1992), Age::try_from(0).unwrap(), Sex::Male)
            .map(|value| f64::from(*value)),
        Some(14_991.03)
    );
}

#[test]
fn loads_deaths_5x1_txt() {
    let table = Deaths::<Single<Year>, Range<Age, 5>>::load(
        include_str!("test_data/Deaths_5x1.txt").as_bytes(),
    )
    .unwrap();

    assert_eq!(
        table
            .query(Year(1990), Age::try_from(2).unwrap(), Sex::Female)
            .map(|value| f64::from(*value)),
        Some(647.0)
    );
}

#[test]
fn loads_e0per_1x10_txt() {
    let table = LifeExpectanciesAtBirth::<Range<Year, 10>>::load(
        include_str!("test_data/E0per_1x10.txt").as_bytes(),
    )
    .unwrap();

    assert_eq!(
        table
            .query(Year(2004), (), Sex::Female)
            .map(ToString::to_string),
        Some("81.83".to_owned())
    );
}

#[test]
fn loads_mx_1x1_txt() {
    let table = CentralDeathRates::<Single<Year>, Single<Age>>::load(
        include_str!("test_data/Mx_1x1.txt").as_bytes(),
    )
    .unwrap();

    assert_eq!(
        table
            .query(Year(1990), Age::try_from(1).unwrap(), Sex::Male)
            .map(|value| f64::from(*value)),
        Some(0.000676)
    );
}
