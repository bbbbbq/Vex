use escargot::CargoBuild;

#[test]
fn test_version_matches_cargo_toml() {
    let expected_version = env!("CARGO_PKG_VERSION");

    let vex_bin = CargoBuild::new()
        .bin("vex")
        .current_release()
        .run()
        .unwrap();

    let output = vex_bin
        .command()
        .arg("--version")
        .output()
        .unwrap();

    assert!(output.status.success(), "vex --version should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_line = stdout.trim();

    let actual_version = version_line
        .split_whitespace()
        .nth(1)
        .expect("Version output should contain version number");

    assert_eq!(
        actual_version, expected_version,
        "Version mismatch: expected '{}', got '{}'",
        expected_version, actual_version
    );
}
