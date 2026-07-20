import os

from human_mortality_database import Country, Session, Sex, TableKind, load_table


def expect_error(error_type, callable_):
    try:
        callable_()
    except error_type:
        return
    except Exception as error:  # noqa: BLE001
        raise AssertionError(f"expected {error_type.__name__}, got {type(error).__name__}: {error}")
    raise AssertionError(f"expected {error_type.__name__}, but no exception was raised")


def test_binding_loads_mx_and_queries_scalar_value() -> None:
    with open("tests/test_data/Mx_1x1.txt", "rb") as handle:
        table = load_table(
            TableKind.CentralDeathRate, handle.read(), year_interval=1, age_interval=1
        )

    value = table.query_scalar(1990, age=1, sex=Sex.Male)
    assert value is not None
    assert abs(value - 0.000676) < 1e-12


def test_binding_table_exposes_country_code_and_last_modified() -> None:
    with open("tests/test_data/Mx_1x1.txt", "rb") as handle:
        table = load_table(
            TableKind.CentralDeathRate, handle.read(), year_interval=1, age_interval=1
        )

    assert table.country_code == "DEUTNP"
    assert table.last_modified == "2022-06-03"


def test_binding_loads_births_and_queries_scalar_value() -> None:
    with open("tests/test_data/Births.txt", "rb") as handle:
        table = load_table(TableKind.Births, handle.read(), year_interval=1)

    value = table.query_scalar(1990, sex=Sex.Female)
    assert value == 440296.0


def test_binding_loads_deaths_1x5_and_queries_scalar_value() -> None:
    with open("tests/test_data/Deaths_1x5.txt", "rb") as handle:
        table = load_table(
            TableKind.Deaths, handle.read(), year_interval=5, age_interval=1
        )

    value = table.query_scalar(1992, age=0, sex=Sex.Male)
    assert value == 14_991.03


def test_binding_loads_deaths_5x1_and_queries_scalar_value() -> None:
    with open("tests/test_data/Deaths_5x1.txt", "rb") as handle:
        table = load_table(
            TableKind.Deaths, handle.read(), year_interval=1, age_interval=5
        )

    value = table.query_scalar(1990, age=2, sex=Sex.Female)
    assert value == 647.0


def test_binding_loads_life_expectancy_and_queries_scalar_value() -> None:
    with open("tests/test_data/E0per_1x10.txt", "rb") as handle:
        table = load_table(TableKind.LifeExpectancyAtBirth, handle.read(), year_interval=10)

    value = table.query_scalar(2004, sex=Sex.Female)
    assert value is not None
    assert abs(value - 81.83) < 1e-9


def test_binding_central_death_rate_returns_none_for_dot_placeholder() -> None:
    data = (
        b"Germany, Death rates (period 1x1),\tLast modified: 03 Jun 2022;  "
        b"Methods Protocol: v6 (2017)\n\n"
        b"Year Age Female Male Total\n"
        b"1990 0 0.10 0.20 0.15\n"
        b"1990 110+ . 1.20 1.20\n"
    )
    table = load_table(TableKind.CentralDeathRate, data, year_interval=1, age_interval=1)

    assert table.query_scalar(1990, age=110, sex=Sex.Female) is None
    male_value = table.query_scalar(1990, age=110, sex=Sex.Male)
    assert male_value is not None
    assert abs(male_value - 1.20) < 1e-12


def test_binding_life_table_query_row_and_handles_undefined_row() -> None:
    data = (
        b"Germany, Life tables (period 1x1), Total\tLast modified: 03 Jun 2022;  "
        b"Methods Protocol: v6 (2017)\n\n"
        b"Year Age mx qx ax lx dx Lx Tx ex\n"
        b"1990 0 0.01 0.02 0.14 100000 2000 99500 7900000 79.0\n"
        b"1990 1 . . . . . . . .\n"
    )
    table = load_table(TableKind.LifeTable, data, year_interval=1, age_interval=1)

    row = table.query_life_table_row(1990, 0)
    assert row is not None
    assert abs(row.ex - 79.0) < 1e-12
    assert abs(row.mx - 0.01) < 1e-12

    assert table.query_life_table_row(1990, 1) is None  # row blanked out with "."
    assert table.query_life_table_row(1991, 0) is None  # year not present at all


def test_binding_query_scalar_on_life_table_raises_value_error() -> None:
    data = (
        b"Germany, Life tables (period 1x1), Total\tLast modified: 03 Jun 2022;  "
        b"Methods Protocol: v6 (2017)\n\n"
        b"Year Age mx qx ax lx dx Lx Tx ex\n"
        b"1990 0 0.01 0.02 0.14 100000 2000 99500 7900000 79.0\n"
    )
    table = load_table(TableKind.LifeTable, data, year_interval=1, age_interval=1)

    expect_error(ValueError, lambda: table.query_scalar(1990, age=0))


def test_binding_query_life_table_row_on_non_life_table_raises_value_error() -> None:
    with open("tests/test_data/Mx_1x1.txt", "rb") as handle:
        table = load_table(
            TableKind.CentralDeathRate, handle.read(), year_interval=1, age_interval=1
        )

    expect_error(ValueError, lambda: table.query_life_table_row(1990, 1))


def test_binding_query_scalar_missing_sex_raises_value_error() -> None:
    with open("tests/test_data/Births.txt", "rb") as handle:
        table = load_table(TableKind.Births, handle.read(), year_interval=1)

    expect_error(ValueError, lambda: table.query_scalar(1990))


def test_binding_query_scalar_missing_age_raises_value_error() -> None:
    with open("tests/test_data/Mx_1x1.txt", "rb") as handle:
        table = load_table(
            TableKind.CentralDeathRate, handle.read(), year_interval=1, age_interval=1
        )

    expect_error(ValueError, lambda: table.query_scalar(1990, sex=Sex.Male))


def test_binding_load_table_rejects_unsupported_interval() -> None:
    with open("tests/test_data/Births.txt", "rb") as handle:
        content = handle.read()

    expect_error(
        ValueError,
        lambda: load_table(TableKind.Births, content, year_interval=3),
    )


def test_binding_session_download_with_country_enum() -> None:
    username = os.environ.get("HMD_USERNAME")
    password = os.environ.get("HMD_PASSWORD")
    if not username or not password:
        print("skipping download integration test: missing HMD credentials")
        return

    session = Session.login(username, password)
    table = session.download(TableKind.Births, Country.Germany, year_interval=1, age_interval=1)
    value = table.query_scalar(1990, sex=Sex.Female)

    assert value is not None
    assert value > 0


if __name__ == "__main__":
    test_binding_loads_mx_and_queries_scalar_value()
    test_binding_table_exposes_country_code_and_last_modified()
    test_binding_loads_births_and_queries_scalar_value()
    test_binding_loads_deaths_1x5_and_queries_scalar_value()
    test_binding_loads_deaths_5x1_and_queries_scalar_value()
    test_binding_loads_life_expectancy_and_queries_scalar_value()
    test_binding_central_death_rate_returns_none_for_dot_placeholder()
    test_binding_life_table_query_row_and_handles_undefined_row()
    test_binding_query_scalar_on_life_table_raises_value_error()
    test_binding_query_life_table_row_on_non_life_table_raises_value_error()
    test_binding_query_scalar_missing_sex_raises_value_error()
    test_binding_query_scalar_missing_age_raises_value_error()
    test_binding_load_table_rejects_unsupported_interval()
    test_binding_session_download_with_country_enum()
    print("python binding integration test passed")
