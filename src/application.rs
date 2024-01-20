mod docker_compose;

use crate::config::Config;
use garde::Validate;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub struct Application<'a> {
  pub name: String,
  pub config: &'a Config,
  values: Values,
}

#[derive(Serialize, Deserialize, Validate)]
struct Values {
  #[garde(required, length(min = 1))]
  path: Option<String>,

  #[garde(skip)]
  repository: Option<String>,
}

pub fn is_exists(conf: &Config, name: &str) -> bool {
  return file_path(conf, name).exists();
}

pub fn find_by<'a>(conf: &'a Config, name: &str) -> Application<'a> {
  let f = file_path(conf, name);
  let s = fs::read_to_string(*f.clone()).expect(&format!("Failed to read {:?}", f));
  let v: Values = toml::from_str(&s).expect(&format!("Failed to load config from {:?}", f));

  if let Err(e) = v.validate(&()) {
    panic!("Invalid application {name} : {e}");
  }

  return Application {
    name: name.to_string(),
    config: conf,
    values: v,
  };
}

pub fn create(conf: &Config, name: &str) {
  let f = file_path(conf, name);
  let mut fs = File::create(*f).unwrap();
  write!(fs, "{}", template()).unwrap();
  fs.flush().unwrap();
}

pub fn template() -> String {
  return r#"# Relative path to the repository directory from not applications directory but root.
path = ""

# URL for the repository
repository = ""
"#
  .to_string();
}

fn file_path(conf: &Config, name: &str) -> Box<PathBuf> {
  return Box::new(conf.app_root().join(format!("{}.toml", name)));
}

impl Application<'_> {
  pub fn root(&self) -> Box<PathBuf> {
    return Box::new(self.config.root().join(&self.values.path.as_ref().unwrap()));
  }

  pub fn dist_root(&self) -> Box<PathBuf> {
    return Box::new(self.config.root().join(".dist").join(&self.name));
  }

  pub fn full_name(&self) -> String {
    match self.config.domain() {
      Some(parent) => format!("{}-{}", parent, self.name.as_str()),
      None => self.name.clone(),
    }
  }

  pub fn domain(&self) -> String {
    let name = self.config.get_alias(self.name.as_str()).unwrap_or(self.name.clone());

    match self.config.domain() {
      Some(parent) => format!("{}.{}", name.as_str(), parent),
      None => name,
    }
  }

  pub fn update(&self, force: bool) {
    let dir = *self.dist_root();
    let mut compose = self.docker_compose();

    if !dir.exists() {
      fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
    }

    if compose.is_up_to_date() && !force {
      return;
    }

    self.clear_dist();
    compose.sync_original();

    let dhcp = compose.create_override();

    self.config.dns().update_config(self, dhcp.dns_config());
  }

  fn docker_compose<'a>(&'a self) -> docker_compose::DockerCompose<'a> {
    return docker_compose::new(&self);
  }

  fn clear_dist(&self) {
    let dir = *self.dist_root();

    if !dir.exists() {
      return;
    }

    fs::read_dir(dir.clone()).expect(&format!("Failed to read {:?}", dir)).for_each(|e| {
      let path = e.unwrap().path();

      fs::remove_file(path.clone()).expect(&format!("Failed to remove {:?}", path));
    });
  }
}
