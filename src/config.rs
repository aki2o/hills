use crate::dns::{self, Dns};
use garde::Validate;
use ipnet::Ipv4Net;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

pub struct Config {
  path: Box<PathBuf>,
  global_values: GlobalValues,
  local_values: LocalValues,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Validate)]
struct GlobalValues {
  #[garde(required, length(min = 1))]
  network_name: Option<String>,
  #[garde(required)]
  subnet: Option<Ipv4Net>,
  #[garde(skip)]
  aliases: Option<BTreeMap<String, String>>,
  #[garde(skip)]
  lima: Option<LimaValues>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Validate)]
pub struct LimaValues {
  #[garde(required)]
  pub cpus: Option<u32>,
  #[garde(required)]
  pub memory: Option<String>,
  #[garde(required)]
  pub disk: Option<String>,
  #[garde(required)]
  pub ssh_port: Option<u32>,
}

#[derive(Serialize, Deserialize, Validate)]
struct LocalValues {
  /// Root domain name of project. It's optional to set if you'd like to make sure the FQDN values are unique in global.
  #[garde(skip)]
  domain: Option<String>,
  /// Relative path to applications directory from root.
  #[garde(required, length(min = 1))]
  app_root: Option<String>,
  /// Root dns server address that's formatted with ipv4.
  #[garde(required)]
  dns: Option<Ipv4Addr>,
}

pub fn create_file(f: Box<PathBuf>, s: String) {
  let error_message = format!("Failed to write {:?}", f);
  let mut fs = File::create(*f.clone()).expect(&error_message);
  write!(fs, "{}", s).expect(&error_message);
  fs.flush().expect(&error_message);
  println!("Saved {:?}", f);
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
  let global_file = global_file_path();

  if !global_dir.exists() {
    fs::create_dir_all(*global_dir.clone()).expect(&format!("Failed to create {:?}", global_dir));
  }

  let v = GlobalValues {
    network_name: Some("hills".to_string()),
    subnet: Some("172.31.0.0/16".parse().unwrap()),
    aliases: None,
    lima: Some(LimaValues {
      cpus: Some(2),
      memory: Some("8GB".to_string()),
      disk: Some("30GB".to_string()),
      ssh_port: Some(2222),
    }),
  };

  if !global_file.exists() {
    create_file(global_file, toml::to_string(&v).unwrap());
  }

  let v = LocalValues {
    domain: Some("".to_string()),
    app_root: Some("applications".to_string()),
    dns: Some(Ipv4Addr::new(8, 8, 8, 8)),
  };

  create_file(local_file_path(root), toml::to_string(&v).unwrap());
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
    return dns::new(
      self.global_values.network_name.as_ref().unwrap().to_string(),
      self.global_values.subnet.unwrap().clone(),
      self.local_values.dns.unwrap().clone(),
    );
  }

  pub fn lima(&self) -> &LimaValues {
    return self.global_values.lima.as_ref().unwrap();
  }

  pub fn application_names(&self) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();

    for entry in fs::read_dir(*self.app_root()).unwrap() {
      let entry = entry.unwrap();
      let path = entry.path();

      if path.is_dir() {
        continue;
      }

      let name = path.file_name().unwrap().to_str().unwrap().strip_suffix(".toml");

      if name.is_none() {
        continue;
      }

      list.push(name.unwrap().to_string());
    }

    return list;
  }

  pub fn resolve(&self, name: &str) -> String {
    let resolved = match &self.global_values.aliases {
      Some(aliases) => match aliases.get(name) {
        Some(v) => v.clone(),
        None => name.to_string(),
      },
      None => name.to_string(),
    };

    if !self.application_names().contains(&resolved) {
      panic!("Not found application : {}", name);
    }

    return resolved;
  }

  pub fn get_alias(&self, original: &str) -> Option<String> {
    match &self.global_values.aliases {
      Some(aliases) => {
        for (k, v) in aliases.iter() {
          if v.eq(original) {
            return Some(k.to_string());
          }
        }

        None
      }
      None => None,
    }
  }

  pub fn set_alias(&mut self, original: &Option<String>, alias: &Option<String>) {
    match original {
      Some(orig) => {
        if let Some(v) = alias {
          if self.global_values.aliases.is_none() {
            self.global_values.aliases = Some(BTreeMap::new());
          }

          self.global_values.aliases.as_mut().unwrap().insert(v.to_string(), orig.to_string());
        } else if let Some(aliases) = self.global_values.aliases.as_mut() {
          let mut filtered: BTreeMap<String, String> = BTreeMap::new();

          for (k, v) in aliases.iter() {
            if v.eq(orig) {
              continue;
            }

            filtered.insert(k.to_string(), v.to_string());
          }

          self.global_values.aliases = Some(filtered);
        }
      }
      None => self.global_values.aliases = None,
    }

    create_file(global_file_path(), toml::to_string(&self.global_values).unwrap());
  }
}
