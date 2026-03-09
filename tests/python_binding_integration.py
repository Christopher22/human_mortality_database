import os

from human_mortality_database import Country, Session, Sex, TableKind, load_table


def test_binding_loads_mx_and_queries_scalar_value() -> None:
    with open("tests/test_data/Mx_1x1.txt", "rb") as handle:
        table = load_table(
            TableKind.CentralDeathRate, handle.read(), year_interval=1, age_interval=1
        )

    value = table.query_scalar(1990, age=1, sex=Sex.Male)
    assert value is not None
    assert abs(value - 0.000676) < 1e-12


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
    test_binding_session_download_with_country_enum()
    print("python binding integration test passed")
