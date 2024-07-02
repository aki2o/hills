mod template;

use crate::application;
use crate::config;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

pub struct Mutagen {
  path: String,
  name: String,
}

pub fn new(path: String) -> Mutagen {
  return Mutagen { path: path, name: "hills".to_string() };
}

impl Mutagen {
  pub fn update(&self) {
    template::ensure_default(&self);

    let mut sync: BTreeMap<String, template::Sync> = BTreeMap::new();

    for entry in fs::read_dir(*self.root()).unwrap() {
      let entry = entry.unwrap();
      let path = entry.path();

      if path.is_file() && (path.ends_with(".yml") || path.ends_with(".yaml")) {
        load(Box::new(path)).sync.into_iter().for_each(|(k, v)| {
          sync.insert(k, v);
        });
      }
    }

    config::current().application_names().into_iter().for_each(|name| {
      let app = application::find_by(&name);

      if !sync.contains_key(&name) {
        sync.insert(
          name.clone(),
          template::Sync {
            alpha: None,
            beta: None,
            mode: None,
            ignore: None,
            permissions: None,
            symlink: None,
          },
        );
      }

      let s = sync.get_mut(&name).unwrap();

      s.alpha = Some(app.root().to_str().unwrap().to_string());
      s.beta = Some(format!("hills:{:?}", PathBuf::from("/usr/src/app").join(app.root().as_ref())));
    });
  }

  fn file_path(&self) -> Box<PathBuf> {
    return Box::new(config::current().dist_root().join("mutagen.yml"));
  }

  fn root(&self) -> Box<PathBuf> {
    let dir = config::current().root().join(self.path);

    if !dir.exists() {
      fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
    }

    return Box::new(dir);
  }
}

fn load(file: Box<PathBuf>) -> template::Yaml {
  let error_message = format!("Failed to load {:?}", file);
  let s = fs::read_to_string(*file).expect(&error_message);

  return serde_yaml::from_str(&s).expect(&error_message);
}
