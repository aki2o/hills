use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::process;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct Network {
  socket: String,
}

pub fn new() -> Network {
  return Network {
    socket: format!("{}/var/run/socket_vmnet", prefix()),
  };
}

fn prefix() -> String {
  return process::Command::new("brew")
    .arg("--prefix")
    .output()
    .expect("Failed to execute brew --prefix")
    .stdout
    .iter()
    .map(|&x| x as char)
    .collect::<String>()
    .trim()
    .to_string();
}
