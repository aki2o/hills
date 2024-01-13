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

impl Application<'_> {
  pub fn root(&self) -> Box<PathBuf> {
    return Box::new(self.config.root().join(&self.values.path.as_ref().unwrap()));
  }

  pub fn is_exists(conf: &Config, name: &str) -> bool {
    return Self::file_path(conf, name).exists();
  }

  pub fn find_by<'a>(conf: &'a Config, name: &str) -> Application<'a> {
    let f = Self::file_path(conf, name);
    let s = fs::read_to_string(*f).expect("Failed to read {f}");
    let v: Values = toml::from_str(&s).expect("Failed to load config from {f}");

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
    let f = Self::file_path(conf, name);
    let mut fs = File::create(*f).unwrap();
    write!(fs, "{}", Self::template()).unwrap();
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

  pub fn docker_compose<'a>(&'a self) -> docker_compose::DockerCompose<'a> {
    return docker_compose::new(&self);
  }

  fn file_path(conf: &Config, name: &str) -> Box<PathBuf> {
    return Box::new(conf.app_root().join(format!("{}.toml", name)));
  }
}
