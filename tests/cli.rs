use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn help_is_available() {
    Command::cargo_bin("forgewatch")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Usage"));
}

#[test]
fn version_is_available() {
    Command::cargo_bin("forgewatch")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("forgewatch"));
}

#[test]
fn analyzes_fixture_and_outputs_json() {
    Command::cargo_bin("forgewatch")
        .unwrap()
        .args(["--json", "tests/fixtures/rust_project"])
        .assert()
        .success()
        .stdout(contains("\"schema_version\": 1"))
        .stdout(contains("\"Rust\""))
        .stdout(contains("\"Cargo\""));
}

#[test]
fn rejects_missing_directory() {
    Command::cargo_bin("forgewatch")
        .unwrap()
        .arg("definitely-not-here")
        .assert()
        .failure();
}
