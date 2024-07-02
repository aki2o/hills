mod network;
mod provision;
mod template;

use crate::config;
use std::fs;
use std::path::PathBuf;

pub struct Lima {
  path: String,
  os: String,
  name: String,
  cpus: u32,
  memory: String,
  disk: String,
  ssh_port: u32,
}

pub fn new(path: String, cpus: u32, memory: String, disk: String, ssh_port: u32) -> Lima {
  return Lima {
    path: path,
    os: "Linux".to_string(),
    name: "hills".to_string(),
    cpus: cpus,
    memory: memory,
    disk: disk,
    ssh_port: ssh_port,
  };
}

impl Lima {
  pub fn os(&self) -> String {
    return self.os.clone();
  }

  pub fn name(&self) -> String {
    return format!("{}-lima", self.name.as_str());
  }

  pub fn start(&self) {
    template::ensure(&self);
    self.update_file();
  }

  fn update_file(&self) {
    let mut yaml = template::load(&self);
    let file = self.file_path();

    yaml.cpus = self.cpus.clone();
    yaml.memory = self.memory.clone();
    yaml.disk = self.disk.clone();
    yaml.ssh.localPort = self.ssh_port.clone();

    config::create_file(file, serde_yaml::to_string(&yaml).unwrap());
  }

  fn file_path(&self) -> Box<PathBuf> {
    return Box::new(config::current().dist_root().join("lima.yml"));
  }

  fn root(&self) -> Box<PathBuf> {
    let dir = config::current().root().join(self.path);

    if !dir.exists() {
      fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
    }

    return Box::new(dir);
  }
}
