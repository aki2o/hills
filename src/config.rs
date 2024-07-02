use crate::dns::{self, Dns};
use crate::lima::{self, Lima};
use crate::mutagen::{self, Mutagen};
use garde::Validate;
use ipnet::Ipv4Net;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Config {
  path: Box<PathBuf>,
  values: Values,
}

#[derive(Serialize, Deserialize, Validate, Debug)]
struct Values {
  /// Relative path to applications directory from root.
  #[garde(required, length(min = 1))]
  app_root: Option<String>,
  #[garde(skip)]
  aliases: Option<BTreeMap<String, String>>,
  #[garde(required)]
  network: Option<NetworkValues>,
  #[garde(required)]
  lima: Option<LimaValues>,
  #[garde(required)]
  mutagen: Option<MutagenValues>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Validate, Debug)]
struct NetworkValues {
  #[garde(required, length(min = 1))]
  root: Option<String>,
  #[garde(required, length(min = 1))]
  name: Option<String>,
  /// Root domain name.
  #[garde(required)]
  domain: Option<String>,
  /// Root dns server address that's formatted with ipv4.
  #[garde(required)]
  dns: Option<Ipv4Addr>,
  #[garde(required)]
  subnet: Option<Ipv4Net>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct LimaValues {
  #[garde(required, length(min = 1))]
  root: Option<String>,
  #[garde(required)]
  cpus: Option<u32>,
  #[garde(required)]
  memory: Option<String>,
  #[garde(required)]
  disk: Option<String>,
  #[garde(required)]
  ssh_port: Option<u32>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct MutagenValues {
  #[garde(required, length(min = 1))]
  root: Option<String>,
}

static INSTANCE: OnceCell<Config> = OnceCell::new();

pub fn current() -> &'static Config {
  return INSTANCE.get().expect("Config file is not found. Run `hills init` to create it.");
}

pub fn try_to_setup_from(root: &Path) {
  let f = file_path(root);

  if !f.exists() {
    return;
  }

  INSTANCE.set(load_from(root)).unwrap();
}

pub fn create(root: &Path) {
  let v = Values {
    app_root: Some("applications".to_string()),
    aliases: None,
    network: Some(NetworkValues {
      root: Some("dns".to_string()),
      name: Some("hills".to_string()),
      domain: Some("local".to_string()),
      subnet: Some("172.31.0.0/16".parse().unwrap()),
      dns: Some(Ipv4Addr::new(8, 8, 8, 8)),
    }),
    lima: Some(LimaValues {
      root: Some("lima".to_string()),
      cpus: Some(2),
      memory: Some("8GB".to_string()),
      disk: Some("30GB".to_string()),
      ssh_port: Some(2222),
    }),
    mutagen: Some(MutagenValues { root: Some("mutagen".to_string()) }),
  };

  create_file(file_path(root), toml::to_string(&v).unwrap());
}

pub fn set_alias(original: &Option<String>, alias: &Option<String>) {
  let root = current().root();
  let mut c = load_from(root.as_ref());

  match original {
    Some(orig) => {
      if let Some(v) = alias {
        if c.values.aliases.is_none() {
          c.values.aliases = Some(BTreeMap::new());
        }

        c.values.aliases.as_mut().unwrap().insert(v.to_string(), orig.to_string());
      } else if let Some(aliases) = c.values.aliases.as_mut() {
        let mut filtered: BTreeMap<String, String> = BTreeMap::new();

        for (k, v) in aliases.iter() {
          if v.eq(orig) {
            continue;
          }

          filtered.insert(k.to_string(), v.to_string());
        }

        c.values.aliases = Some(filtered);
      }
    }
    None => c.values.aliases = None,
  }

  create_file(file_path(root.as_ref()), toml::to_string(&c.values).unwrap());
}

pub fn create_file(f: Box<PathBuf>, s: String) {
  let error_message = format!("Failed to write {:?}", f);
  let mut fs = File::create(*f.clone()).expect(&error_message);
  write!(fs, "{}", s).expect(&error_message);
  fs.flush().expect(&error_message);
  println!("Saved {:?}", f);
}

fn file_path(root: &Path) -> Box<PathBuf> {
  return Box::new(root.join("Hills.toml"));
}

fn load_from(root: &Path) -> Config {
  let f = file_path(root);
  let s = fs::read_to_string(*f.clone()).expect(&format!("Failed to read {:?}", f));
  let values: Values = toml::from_str(&s).expect(&format!("Failed to load config from {:?}", f));

  if let Err(e) = values.validate(&()) {
    panic!("Invalid config {:?} : {e}", f);
  }

  return Config { path: f, values: values };
}

impl Config {
  pub fn root(&self) -> Box<PathBuf> {
    return Box::new(self.path.parent().unwrap().to_path_buf());
  }

  pub fn dist_root(&self) -> Box<PathBuf> {
    let dir = self.root().join(".dist");

    if !dir.exists() {
      fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
    }

    return Box::new(dir);
  }

  pub fn domain(&self) -> String {
    return self.values.network.unwrap().domain.unwrap().trim().to_string();
  }

  pub fn app_root(&self) -> Box<PathBuf> {
    return Box::new(self.path.parent().unwrap().join(&self.values.app_root.unwrap()));
  }

  pub fn dns(&self) -> Dns {
    let c = self.values.network.unwrap();

    return dns::new(c.root.unwrap().to_string(), c.name.unwrap().to_string(), c.domain.unwrap().to_string(), c.subnet.unwrap().clone(), c.dns.unwrap().clone());
  }

  pub fn lima(&self) -> Lima {
    let c = self.values.lima.unwrap();

    return lima::new(c.root.unwrap(), c.cpus.unwrap(), c.memory.unwrap(), c.disk.unwrap(), c.ssh_port.unwrap());
  }

  pub fn mutagen(&self) -> Mutagen {
    let c = self.values.mutagen.unwrap();

    return mutagen::new(c.root.unwrap());
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
    let resolved = match &self.values.aliases {
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
    match &self.values.aliases {
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
}
