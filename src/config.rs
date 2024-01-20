use crate::dns::{self, Dns};
use garde::Validate;
use ipnet::Ipv4Net;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Config {
  path: Box<PathBuf>,
  global_values: GlobalValues,
  local_values: LocalValues,
}

#[derive(Serialize, Deserialize, Validate)]
struct GlobalValues {
  #[garde(required, length(min = 1))]
  network_name: Option<String>,
  #[garde(required)]
  subnet: Option<Ipv4Net>,
}

#[derive(Serialize, Deserialize, Validate)]
struct LocalValues {
  /// The root domain name of project. It's optional to set if you'd like to make sure the FQDN values are unique in global.
  #[garde(skip)]
  domain: Option<String>,
  /// The relative path to applications directory from root.
  #[garde(required, length(min = 1))]
  app_root: Option<String>,
}

pub fn create_file(f: Box<PathBuf>, s: String) {
  let error_message = format!("Failed to write {:?}", f);
  let mut fs = File::create(*f.clone()).expect(&error_message);
  write!(fs, "{}", s).expect(&error_message);
  fs.flush().expect(&error_message);
  println!("Created {:?}", f);
}

pub fn global_dir_path() -> Box<PathBuf> {
  return Box::new(dirs::home_dir().unwrap().join(".hills"));
}

fn global_file_path() -> Box<PathBuf> {
  return Box::new(global_dir_path().join("config.toml"));
}

fn local_file_path(root: &Path) -> Box<PathBuf> {
  return Box::new(root.join("Hills.toml"));
}

impl Config {
  pub fn root(&self) -> Box<PathBuf> {
    return Box::new(self.path.parent().unwrap().to_path_buf());
  }

  pub fn domain(&self) -> Option<String> {
    match &self.local_values.domain {
      Some(v) => {
        if v.trim().eq("") {
          None
        } else {
          Some(v.trim().to_string())
        }
      }
      None => None,
    }
  }

  pub fn app_root(&self) -> Box<PathBuf> {
    match &self.local_values.app_root {
      Some(v) => Box::new(self.path.parent().unwrap().join(&v)),
      None => panic!("Invalid config: app_root is not set."),
    }
  }

  pub fn dns(&self) -> Dns {
    return dns::new(self.global_values.network_name.as_ref().unwrap().to_string(), self.global_values.subnet.unwrap().clone());
  }

  pub fn load_from(root: &Path) -> Config {
    let f = local_file_path(root);
    let s = fs::read_to_string(*f.clone()).expect(&format!("Failed to read {:?}", f));
    let local_values: LocalValues = toml::from_str(&s).expect(&format!("Failed to load config from {:?}", f));
    if let Err(e) = local_values.validate(&()) {
      panic!("Invalid config {:?} : {e}", f);
    }

    let f = global_file_path();
    let s = fs::read_to_string(*f.clone()).expect(&format!("Failed to read {:?}", f));
    let global_values: GlobalValues = toml::from_str(&s).expect(&format!("Failed to load config from {:?}", f));
    if let Err(e) = global_values.validate(&()) {
      panic!("Invalid config {:?} : {e}", f);
    }

    return Config {
      path: local_file_path(root),
      global_values: global_values,
      local_values: local_values,
    };
  }

  pub fn create(root: &Path) {
    let global_dir = global_dir_path();

    if !global_dir.exists() {
      fs::create_dir_all(*global_dir.clone()).expect(&format!("Failed to create {:?}", global_dir));
    }

    let v = GlobalValues {
      network_name: Some("hills".to_string()),
      subnet: Some("172.31.0.0/16".parse().unwrap()),
    };

    create_file(global_file_path(), toml::to_string(&v).unwrap());

    let v = LocalValues {
      domain: Some("".to_string()),
      app_root: Some("applications".to_string()),
    };

    create_file(local_file_path(root), toml::to_string(&v).unwrap());
  }
}
