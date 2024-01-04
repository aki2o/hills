use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs::{self, File};
use std::process::Command;

mod functions;

// #[test]
// fn new_if_the_dir_absent() -> Result<(), Box<dyn std::error::Error>> {
//     let dir = functions::setup_cwd();
//     functions::setup_config(&dir);

//     let mut cmd = Command::cargo_bin("hills")?;

//     cmd.arg("new", "foo").assert().success();

//     let f = dir.path().join("applications").join("foo.toml");
//     assert!(f.exists());

//     let s = fs::read_to_string(f)?;

//     let expected = "app_root = \"applications\"\n";

//     assert_eq!(s, expected);

//     Ok(())
// }
