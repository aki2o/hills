use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fs;
use std::path::PathBuf;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct Provision {
  mode: String,
  script: String,
}

pub fn new(script: String, as_user: bool) -> Provision {
  let mode = if as_user { "user" } else { "system" };

  return Provision { mode: mode.to_string(), script: script };
}

pub fn from(file: Box<PathBuf>) -> Provision {
  let script = fs::read_to_string(*file).expect(&format!("Failed to read {:?}", file));

  return Provision { mode: "system".to_string(), script: script };
}

pub fn dispatch(from: Box<PathBuf>, to: Box<PathBuf>) -> Provision {
  let body = fs::read_to_string(*from).expect(&format!("Failed to read {:?}", from));

  let script = format!(
    r#"#!/bin/bash
cat <<'EOF' > {}
{}
EOF
"#,
    to.display(),
    body
  );

  return Provision { mode: "system".to_string(), script: script };
}
