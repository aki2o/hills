use crate::config;
use crate::lima;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fs;
use std::path::PathBuf;
use std::process;

use super::Lima;

#[allow(non_snake_case)]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct Yaml {
  vmType: String,
  pub cpus: u32,
  pub memory: String,
  pub disk: String,
  arch: String,
  images: Vec<Image>,
  containerd: Containerd,
  pub ssh: Ssh,
  provision: Option<Vec<lima::provision::Provision>>,
  portForwards: Vec<PortForward>,
  networks: Vec<lima::network::Network>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct Image {
  location: String,
  arch: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct Containerd {
  system: bool,
  user: bool,
}

#[allow(non_snake_case)]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct Ssh {
  pub localPort: u32,
  forwardAgent: bool,
  loadDotSSHPubKeys: bool,
}

#[allow(non_snake_case)]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct PortForward {
  guestPort: u32,
  hostPort: u32,
}

pub fn ensure(lima: &Lima) {
  let file = path(lima);

  if !file.exists() {
    create(lima);
  }
}

pub fn load(lima: &Lima) -> Yaml {
  let file = path(lima);
  let error_message = format!("Failed to load {:?}", file);
  let s = fs::read_to_string(*file).expect(&error_message);

  return serde_yaml::from_str(&s).expect(&error_message);
}

fn path(lima: &Lima) -> Box<PathBuf> {
  return Box::new(lima.root().join("template.yml"));
}

fn create(lima: &Lima) {
  let arch = arch();
  let yaml = Yaml {
    vmType: "qemu".to_string(),
    cpus: 2,
    memory: "8GB".to_string(),
    disk: "30GB".to_string(),
    arch: arch.clone(),
    images: vec![Image {
      location: image_url_for(arch.as_str()),
      arch: arch.clone(),
    }],
    containerd: Containerd { system: false, user: false },
    ssh: Ssh {
      localPort: 2222,
      forwardAgent: true,
      loadDotSSHPubKeys: true,
    },
    provision: None,
    portForwards: vec![PortForward { guestPort: 53, hostPort: 53 }],
    networks: vec![lima::network::new()],
  };

  config::create_file(path(lima), serde_yaml::to_string(&yaml).unwrap());
}

fn arch() -> String {
  let value = process::Command::new("uname")
    .arg("-m")
    .output()
    .expect("Failed to execute uname")
    .stdout
    .iter()
    .map(|&x| x as char)
    .collect::<String>()
    .trim()
    .to_string();

  return match value.as_str() {
    "x86_64" => "x86_64".to_string(),
    "arm64" => "aarch64".to_string(),
    _ => panic!("Unsupported architecture: {}", value),
  };
}

fn image_url_for(arch: &str) -> String {
  match arch {
    "x86_64" => "https://cloud-images.ubuntu.com/releases/22.04/release/ubuntu-22.04-server-cloudimg-amd64.img".to_string(),
    "aarch64" => "https://cloud-images.ubuntu.com/releases/22.04/release/ubuntu-22.04-server-cloudimg-arm64.img".to_string(),
    _ => panic!("Unsupported architecture: {}", arch),
  }
}
