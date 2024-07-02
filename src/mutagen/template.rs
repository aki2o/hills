use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::config;

use super::Mutagen;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct Yaml {
  pub sync: BTreeMap<String, Sync>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct Sync {
  pub alpha: Option<String>,
  pub beta: Option<String>,
  pub mode: Option<String>,
  pub ignore: Option<Ignore>,
  pub permissions: Option<Permissions>,
  pub symlink: Option<SymLink>,
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

pub fn ensure_default(mutagen: &Mutagen) {
  let file = mutagen.root().join("default.yml");

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
