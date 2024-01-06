use crate::application::Application;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

pub struct DockerCompose<'a> {
    app: &'a Application<'a>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Yaml {
    services: Option<BTreeMap<String, Service>>,
    networks: Option<BTreeMap<String, Network>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
    container_name: Option<String>,
    dns: Option<Vec<String>>,
    networks: Option<(Vec<String>, BTreeMap<String, Network>)>,
    tty: Option<String>,
    stdin_open: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Network {
    external: Option<bool>,
    ipv4_address: Option<String>,
    aliases: Option<Vec<String>>,
}

impl DockerCompose<'_> {
    pub fn new<'a>(app: &'a Application<'a>) -> DockerCompose<'a> {
        return DockerCompose { app: app };
    }

    pub fn sync(&self) {
        let dir = *self.dist_root();

        if !dir.exists() {
            fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
        }

        let new_file = self.sync_original();

        if new_file == None {
            return;
        }

        let new_file = *new_file.unwrap();

        fs::read_dir(dir.clone())
            .expect(&format!("Failed to read {:?}", dir))
            .for_each(|e| {
                let path = e.unwrap().path();

                if path.is_file() && path != new_file {
                    fs::remove_file(path.clone()).expect(&format!("Failed to remove {:?}", path));
                }
            });

        self.create_override(Box::new(new_file));
    }

    fn dist_root(&self) -> Box<PathBuf> {
        return Box::new(self.app.config.root().join(".dist").join(&self.app.name));
    }

    fn hash_of(&self, file: Box<PathBuf>) -> String {
        Command::new("shasum")
            .arg("-a")
            .arg("256")
            .arg(file.as_os_str())
            .output()
            .expect(&format!(
                "Failed to execute shasum -a 256 {:?}",
                file.as_os_str()
            ))
            .stdout
            .iter()
            .map(|&x| x as char)
            .collect::<String>()
            .split(" ")
            .collect::<Vec<&str>>()[0]
            .to_string()
    }

    fn sync_original(&self) -> Option<Box<PathBuf>> {
        let dir = *self.dist_root();
        let orig_path = self.app.root().join("docker-compose.yml");
        let orig_hash = self.hash_of(Box::new(orig_path.clone()));
        let path = dir.join(format!("{}.yml", orig_hash));

        if path.exists() {
            return None;
        }

        let read_error_message = format!("Failed to read {:?}", orig_path);
        let orig_file = File::open(orig_path.clone()).expect(&read_error_message);
        let lines = io::BufReader::new(orig_file).lines();

        let write_error_message = format!("Failed to write {:?}", path);
        let file = File::create(path.clone()).expect(&write_error_message);
        let mut writer = io::BufWriter::new(file);

        let mut on_ports = false;
        for line in lines {
            let l = line.expect(&read_error_message);

            if l.trim().starts_with("#") {
                continue;
            }

            if on_ports {
                if l.trim().starts_with("-") {
                    // https://docs.docker.com/compose/environment-variables/env-file/
                    let re = Regex::new(r#"^(\s+-\s+['"]?).+:([^-?+])"#).unwrap();

                    // Removing a part of exposing port to host
                    writeln!(writer, "{}", re.replace(&l, "$1$2")).expect(&write_error_message);
                } else {
                    on_ports = false;
                    writeln!(writer, "{}", l).expect(&write_error_message);
                }
            } else {
                if l.trim().starts_with("ports:") {
                    on_ports = true;

                    if l.trim() != "ports:" {
                        // TODO: support `ports: [...]`
                        panic!(
                            "Failed to read {:?} : unsupport `ports: [...]` format",
                            path
                        )
                    }
                }
                writeln!(writer, "{}", l).expect(&write_error_message);
            }
        }

        writer.flush().expect(&write_error_message);

        return Some(Box::new(path));
    }

    fn create_override(&self, file: Box<PathBuf>) {
        let mut yaml = self.load(file);

        let mut services = yaml.services.unwrap_or(BTreeMap::new());
        let mut networks = yaml.networks.unwrap_or(BTreeMap::new());

        let mut service = services
            .get(&self.app.name)
            .unwrap_or(&Service {
                container_name: None,
                dns: None,
                networks: None,
                tty: None,
                stdin_open: None,
            })
            .clone();
    }

    fn load(&self, file: Box<PathBuf>) -> Yaml {
        let error_message = format!("Failed to load {:?}", file);
        let s = fs::read_to_string(*file.clone()).expect(&error_message);
        return serde_yaml::from_str(&s).expect(&error_message);
    }
}
