use crate::application;
use crate::config;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

pub struct Mutagen {
  pub name: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct Yaml {
  sync: BTreeMap<String, Sync>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct Sync {
  alpha: Option<String>,
  beta: Option<String>,
  mode: Option<String>,
  ignore: Option<Ignore>,
  permissions: Option<Permissions>,
  symlink: Option<SymLink>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct Ignore {
  vcs: Option<bool>,
  paths: Option<Vec<String>>,
}

#[allow(non_snake_case)]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct Permissions {
  defaultFileMode: Option<String>,
  defaultDirectoryMode: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
struct SymLink {
  mode: Option<String>,
}

pub fn new() -> Mutagen {
  return Mutagen { name: "hills".to_string() };
}

pub fn root() -> Box<PathBuf> {
  let dir = config::global_dir_path().join("mutagen");

  if !dir.exists() {
    fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
  }

  return Box::new(dir);
}

fn file_path() -> Box<PathBuf> {
  let dir = config::global_dir_path().join(".dist");

  if !dir.exists() {
    fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
  }

  return Box::new(dir.join("mutagen.yml"));
}

pub fn update(conf: &config::Config) {
  ensure_default();

  let mut yaml = Yaml { sync: BTreeMap::new() };

  for entry in fs::read_dir(*root()).unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();

    if path.is_file() && (path.ends_with(".yml") || path.ends_with(".yaml")) {
      load(Box::new(path)).sync.into_iter().for_each(|(k, v)| {
        yaml.sync.insert(k, v);
      });
    }
  }

  conf.application_names().into_iter().for_each(|name| {
    let app = application::find_by(conf, &name);

    if !yaml.sync.contains_key(&name) {
      yaml.sync.insert(
        name.clone(),
        Sync {
          alpha: None,
          beta: None,
          mode: None,
          ignore: None,
          permissions: None,
          symlink: None,
        },
      );
    }

    let sync = yaml.sync.get_mut(&name).unwrap();

    sync.alpha = Some(app.root().to_str().unwrap().to_string());
    sync.beta = Some(format!("hills:{:?}", PathBuf::from("/usr/src/app").join(app.root().as_ref())));
  });
}

fn load(file: Box<PathBuf>) -> Yaml {
  let error_message = format!("Failed to load {:?}", file);
  let s = fs::read_to_string(*file).expect(&error_message);

  return serde_yaml::from_str(&s).expect(&error_message);
}

fn ensure_default() {
  let file = root().join("default.yml");

  if file.exists() {
    return;
  }

  let mut sync: BTreeMap<String, Sync> = BTreeMap::new();

  let ignore = Ignore {
    vcs: Some(true),
    paths: Some(vec![
      // trash and backup
      ".DS_Store".to_string(),
      "._*".to_string(),
      "*~".to_string(),
      "*.sw[a-p]".to_string(),
      // log
      "*.log*".to_string(),
      // Ruby
      "/tmp/cache/".to_string(),
      "/log/".to_string(),
      "vendor/bundle/".to_string(),
      // Node
      "node_modules/".to_string(),
      ".pnpm-store/".to_string(),
      ".next/".to_string(),
    ]),
  };

  sync.insert(
    "defaults".to_string(),
    Sync {
      alpha: None,
      beta: None,
      mode: None,
      ignore: Some(ignore),
      permissions: Some(Permissions {
        defaultFileMode: Some("0644".to_string()),
        defaultDirectoryMode: Some("0755".to_string()),
      }),
      symlink: Some(SymLink { mode: Some("ignore".to_string()) }),
    },
  );

  let yaml = Yaml { sync: sync };

  config::create_file(Box::new(file), serde_yaml::to_string(&yaml).unwrap());
}

impl Mutagen {}
