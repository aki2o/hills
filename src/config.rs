use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Config {
    path: Box<PathBuf>,
    values: Values,
}

#[derive(Serialize, Deserialize)]
struct Values {
    /// The relative path to applications directory from root.
    app_root: String,
}

impl Config {
    pub fn app_root(&self) -> Box<PathBuf> {
        return Box::new(self.path.parent().unwrap().join(&self.values.app_root));
    }

    pub fn load_from(root: &Path) -> Config {
        let f = Self::file_path(root);
        let s = fs::read_to_string(*f).expect("Failed to read {f}");
        let v = toml::from_str(&s).expect("Failed to load config from {f}");

        return Config {
            path: Self::file_path(root),
            values: v,
        };
    }

    pub fn create(root: &Path) {
        let v = Values {
            app_root: "applications".to_string(),
        };

        let f = Self::file_path(root);
        let s = toml::to_string(&v).unwrap();
        let mut fs = File::create(*f).unwrap();
        write!(fs, "{}", s).unwrap();
        fs.flush().unwrap();
    }

    fn file_path(root: &Path) -> Box<PathBuf> {
        return Box::new(root.join("Hills.toml"));
    }
}
