#![allow(dead_code)]

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use uuid::Uuid;

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

pub fn at_path(path: &Path) -> String {
    format!("@{}", path.display())
}

pub fn run_json(args: &[&str]) -> Value {
    let output = assert_cmd::cargo::cargo_bin_cmd!()
        .arg("--json")
        .args(args)
        .output()
        .expect("failed to run jwt-tester");
    assert!(
        output.status.success(),
        "command failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("invalid JSON output")
}

pub fn assert_exit(args: &[&str], code: i32) {
    assert_cmd::cargo::cargo_bin_cmd!()
        .args(args)
        .assert()
        .failure()
        .code(code);
}

pub fn encode_token(args: &[&str]) -> String {
    let json = run_json(args);
    json["data"]["token"]
        .as_str()
        .expect("missing token")
        .to_string()
}

pub struct TestVault {
    dir: TempDir,
    service: String,
    passphrase: String,
    keychain_dir: PathBuf,
}

impl TestVault {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("temp dir");
        let keychain_dir = dir.path().join("keychain");
        Self {
            dir,
            service: format!("jwt-tester-test-{}", Uuid::new_v4()),
            passphrase: format!("passphrase-{}", Uuid::new_v4()),
            keychain_dir,
        }
    }

    pub fn cmd(&self) -> Command {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!();
        cmd.arg("--data-dir")
            .arg(self.dir.path())
            .env("JWT_TESTER_KEYCHAIN_SERVICE", &self.service)
            .env("JWT_TESTER_KEYCHAIN_BACKEND", "file")
            .env("JWT_TESTER_KEYCHAIN_PASSPHRASE", &self.passphrase)
            .env("JWT_TESTER_DOCKER", "1")
            .env("JWT_TESTER_DOCKER_TEST", "1")
            .env("JWT_TESTER_KEYCHAIN_DIR", self.keychain_dir.as_os_str());
        cmd
    }

    pub fn run_json(&self, args: &[&str]) -> Value {
        let output = self
            .cmd()
            .arg("--json")
            .args(args)
            .output()
            .expect("failed to run jwt-tester");
        assert!(
            output.status.success(),
            "command failed: stdout={}, stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).expect("invalid JSON output")
    }

    pub fn assert_exit(&self, args: &[&str], code: i32) {
        self.cmd().args(args).assert().failure().code(code);
    }
}
