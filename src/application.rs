mod runner;
mod synchronizer;

use crate::config;
use garde::Validate;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub struct Application {
  pub name: String,
  values: Values,
}

#[derive(Serialize, Deserialize, Validate)]
struct Values {
  #[garde(required, length(min = 1))]
  path: Option<String>,

  #[garde(skip)]
  repository: Option<String>,
}

pub fn is_exists(name: &str) -> bool {
  return file_path(name).exists();
}

pub fn find_by(name: &str) -> Application {
  let f = file_path(name);
  let s = fs::read_to_string(*f.clone()).expect(&format!("Failed to read {:?}", f));
  let v: Values = toml::from_str(&s).expect(&format!("Failed to load config from {:?}", f));

  if let Err(e) = v.validate(&()) {
    panic!("Invalid application {name} : {e}");
  }

  return Application { name: name.to_string(), values: v };
}

pub fn create(name: &str) {
  let f = file_path(name);
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

fn file_path(name: &str) -> Box<PathBuf> {
  let c = config::current();

  return Box::new(c.app_root().join(format!("{}.toml", c.resolve(name))));
}

impl Application {
  pub fn root(&self) -> Box<PathBuf> {
    return Box::new(config::current().root().join(&self.values.path.as_ref().unwrap()));
  }

  pub fn dist_root(&self) -> Box<PathBuf> {
    return Box::new(config::current().root().join(".dist").join(&self.name));
  }

  pub fn name(&self) -> &str {
    return self.name.as_str();
  }

  pub fn domain(&self) -> String {
    let name = config::current().get_alias(self.name.as_str()).unwrap_or(self.name.clone());

    return format!("{}.{}", name.as_str(), config::current().dns().domain());
  }

  pub fn update(&self, force: bool) {
    let dir = self.dist_root();
    let mut synchronizer = synchronizer::new(&self);

    if !dir.exists() {
      fs::create_dir_all(*dir.clone()).expect(&format!("Failed to create {:?}", dir));
    }

    if synchronizer.is_up_to_date() && !force {
      return;
    }

    self.clear_dist();
    synchronizer.perform();
  }

  pub fn print(&self) {
    self.make_runner().ps();
  }

  fn clear_dist(&self) {
    self.docker_compose_paths().iter().for_each(|p| {
      fs::remove_file(*p.clone()).expect(&format!("Failed to remove {:?}", p));
    });
  }

  fn make_runner(&self) -> Box<runner::Runner> {
    return Box::new(runner::new(self, self.docker_compose_paths()));
  }

  fn docker_compose_paths(&self) -> Vec<Box<PathBuf>> {
    let dir = self.dist_root();
    let mut list: Vec<Box<PathBuf>> = vec![];

    if dir.exists() {
      fs::read_dir(*dir.clone()).expect(&format!("Failed to read {:?}", dir)).for_each(|f| {
        let path = f.unwrap().path();

        if path.is_file() {
          list.push(Box::new(path));
        }
      });
    }

    return list;
  }
}
