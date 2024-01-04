use hills::config::Config;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use tempdir::TempDir;

pub fn setup_cwd() -> TempDir {
    let dir = TempDir::new("hills").unwrap();
    let _ = env::set_current_dir(&dir);
    dir
}

pub fn setup_config(dir: &TempDir) {
    let f = dir.path().join("Hills.toml");
    let s = "app_root = \"applications\"\n";

    let mut fs = File::create(f).unwrap();
    write!(fs, "{}", s).unwrap();
    fs.flush().unwrap();
}
