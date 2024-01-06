use garde::Validate;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Config {
    path: Box<PathBuf>,
    local_values: LocalValues,
}

#[derive(Serialize, Deserialize, Validate)]
struct GlobalValues {
    #[garde(required)]
    subnet: Option<String>,
}

#[derive(Serialize, Deserialize, Validate)]
struct LocalValues {
    /// The relative path to applications directory from root.
    #[garde(required, length(min = 1))]
    app_root: Option<String>,
}

impl Config {
    pub fn root(&self) -> Box<PathBuf> {
        return Box::new(self.path.parent().unwrap().to_path_buf());
    }

    pub fn app_root(&self) -> Box<PathBuf> {
        match &self.local_values.app_root {
            Some(v) => Box::new(self.path.parent().unwrap().join(&v)),
            None => panic!("Invalid config: app_root is not set."),
        }
    }

    pub fn load_from(root: &Path) -> Config {
        let f = Self::local_file_path(root);
        let s = fs::read_to_string(*f).expect("Failed to read {f}");
        let v: LocalValues = toml::from_str(&s).expect("Failed to load config from {f}");

        if let Err(e) = v.validate(&()) {
            panic!("Invalid config {:?} : {e}", Self::local_file_path(root));
        }

        return Config {
            path: Self::local_file_path(root),
            local_values: v,
        };
    }

    pub fn create(root: &Path) {
        let global_dir = Self::global_dir_path();

        if !global_dir.exists() {
            fs::create_dir_all(*global_dir.clone())
                .expect(&format!("Failed to create {:?}", global_dir));
        }

        let v = GlobalValues {
            subnet: Some("172.31.0.0/16".to_string()),
        };

        Self::create_file(Self::global_file_path(), toml::to_string(&v).unwrap());

        let v = LocalValues {
            app_root: Some("applications".to_string()),
        };

        Self::create_file(Self::local_file_path(root), toml::to_string(&v).unwrap());
    }

    fn global_dir_path() -> Box<PathBuf> {
        return Box::new(PathBuf::from("~/.hills"));
    }

    fn global_file_path() -> Box<PathBuf> {
        return Box::new(Self::global_dir_path().join("config.toml"));
    }

    fn local_file_path(root: &Path) -> Box<PathBuf> {
        return Box::new(root.join("Hills.toml"));
    }

    fn create_file(f: Box<PathBuf>, s: String) {
        let error_message = format!("Failed to write {:?}", f);
        let mut fs = File::create(*f).expect(&error_message);
        write!(fs, "{}", s).expect(&error_message);
        fs.flush().expect(&error_message);
    }
}
