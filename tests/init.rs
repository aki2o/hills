use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs::{self, File};
use std::process::Command;

mod functions;

#[test]
fn init_if_the_file_absent() -> Result<(), Box<dyn std::error::Error>> {
    let dir = functions::setup_cwd();

    let mut cmd = Command::cargo_bin("hills")?;

    cmd.arg("init").assert().success();

    let f = dir.path().join("Hills.toml");
    assert!(f.exists());

    let s = fs::read_to_string(f)?;

    let expected = "app_root = \"applications\"\n";

    assert_eq!(s, expected);

    Ok(())
}
