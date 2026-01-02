#[test]
fn completion_bash_outputs_script() {
    let output = assert_cmd::cargo::cargo_bin_cmd!()
        .args(["completion", "bash"])
        .output()
        .expect("completion");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty());
    assert!(stdout.contains("jwt-tester"));
}
