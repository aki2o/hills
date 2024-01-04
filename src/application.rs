use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub struct Application<'a> {
    pub name: String,
    config: &'a Config,
    values: Values,
}

#[derive(Serialize, Deserialize)]
struct Values {
    /// The relative path to the repository directory from not applications directory but root.
    path: String,
    repository: String,
}

impl Application<'_> {
    pub fn is_exists(conf: &Config, name: &str) -> bool {
        return Self::file_path(conf, name).exists();
    }

    pub fn find_by<'a>(conf: &'a Config, name: &str) -> Application<'a> {
        let f = Self::file_path(conf, name);
        let s = fs::read_to_string(*f).expect("Failed to read {f}");
        let v = toml::from_str(&s).expect("Failed to load config from {f}");

        return Application {
            name: name.to_string(),
            config: conf,
            values: v,
        };
    }

    pub fn create(conf: &Config, name: &str) {
        let v = Values {
          path: "# Relative path to the repository directory from not applications directory but root.".to_string(),
          repository: "# URL for the repository".to_string(),
        };

        let f = Self::file_path(conf, name);
        let s = toml::to_string(&v).unwrap();
        let mut fs = File::create(*f).unwrap();
        write!(fs, "{}", s).unwrap();
        fs.flush().unwrap();
    }

    fn file_path(conf: &Config, name: &str) -> Box<PathBuf> {
        return Box::new(conf.app_root().join(format!("{}.toml", name)));
    }
}
