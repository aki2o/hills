mod network;

use crate::config;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fs;
use std::path::PathBuf;
use std::process;

pub struct Lima {
  pub os: String,
  pub name: String,
}

#[allow(non_snake_case)]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct Yaml {
  vmType: String,
  cpus: u32,
  memory: String,
  disk: String,
  arch: String,
  images: Vec<Image>,
  containerd: Containerd,
  ssh: Ssh,
  provision: Vec<Provision>,
  portForwards: Vec<PortForward>,
  networks: Vec<network::Network>,
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
  localPort: u32,
  forwardAgent: bool,
  loadDotSSHPubKeys: bool,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct Provision {
  mode: String,
  script: String,
}

#[allow(non_snake_case)]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct PortForward {
  guestPort: u32,
  hostPort: u32,
}

pub fn new() -> Lima {
  return Lima {
    os: "Linux".to_string(),
    name: "hills".to_string(),
  };
}

pub fn root() -> Box<PathBuf> {
  let dir = config::global_dir_path().join("lima");

  if !dir.exists() {
    fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
  }

  return Box::new(dir);
}

pub fn template_path() -> Box<PathBuf> {
  return Box::new(root().join("template.yml"));
}

fn file_path() -> Box<PathBuf> {
  let dir = config::global_dir_path().join(".dist");

  if !dir.exists() {
    fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
  }

  return Box::new(dir.join("lima.yml"));
}

fn ensure_template() {
  let file = template_path();

  if file.exists() {
    return;
  }

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
    provision: vec![],
    portForwards: vec![PortForward { guestPort: 53, hostPort: 53 }],
    networks: vec![network::new()],
  };

  config::create_file(file, serde_yaml::to_string(&yaml).unwrap());
}

fn load_template() -> Yaml {
  let file = template_path();
  let error_message = format!("Failed to load {:?}", file);
  let s = fs::read_to_string(*file).expect(&error_message);

  return serde_yaml::from_str(&s).expect(&error_message);
}

fn update_file(conf: &config::Config) {
  let mut yaml = load_template();
  let file = file_path();
  let lima = conf.lima();

  yaml.cpus = lima.cpus.unwrap().clone();
  yaml.memory = lima.memory.as_ref().unwrap().clone();
  yaml.disk = lima.disk.as_ref().unwrap().clone();
  yaml.ssh.localPort = lima.ssh_port.unwrap().clone();

  config::create_file(file, serde_yaml::to_string(&yaml).unwrap());
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

impl Lima {
  pub fn os(&self) -> String {
    return self.os.clone();
  }

  pub fn name(&self) -> String {
    return format!("{}-lima", self.name.as_str());
  }

  pub fn start(&self, conf: &config::Config) {
    ensure_template();
    update_file(conf);
  }
}
