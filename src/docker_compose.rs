pub mod command;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::prelude::*;
use std::net::Ipv4Addr;
use std::path::PathBuf;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct Yaml {
  pub version: Option<String>,
  pub services: Option<BTreeMap<String, Service>>,
  pub networks: Option<BTreeMap<String, Network>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
  pub container_name: Option<String>,
  pub build: Option<ServiceBuild>,
  pub volumes: Option<Vec<String>>,
  pub ports: Option<Vec<String>>,
  pub networks: Option<ServiceNetworkable>,
  pub dns: Option<Vec<Ipv4Addr>>,
  pub tty: Option<String>,
  pub stdin_open: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceBuild {
  pub context: Option<String>,
  pub dockerfile: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServiceNetworkable {
  List(Vec<String>),
  Map(BTreeMap<String, Network>),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct Network {
  pub external: Option<bool>,
  pub ipv4_address: Option<Ipv4Addr>,
  pub aliases: Option<Vec<String>>,
}

pub fn load(file: Box<PathBuf>) -> Yaml {
  let error_message = format!("Failed to load {:?}", file);
  let s = fs::read_to_string(*file).expect(&error_message);
  return serde_yaml::from_str(&s).expect(&error_message);
}

impl Yaml {
  pub fn save(&self, file: Box<PathBuf>) {
    let error_message = format!("Failed to write {:?}", file);
    let mut f = File::create(*file).expect(&error_message);
    write!(f, "{}", serde_yaml::to_string(&self).unwrap()).expect(&error_message);
    f.flush().expect(&error_message);
  }
}
