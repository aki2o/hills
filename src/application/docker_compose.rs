use crate::application::Application;
use crate::dhcp;
use crate::dns;
use crate::docker_compose;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

pub struct DockerCompose<'a> {
  app: &'a Application<'a>,
}

pub fn new<'a>(app: &'a Application<'a>) -> DockerCompose<'a> {
  return DockerCompose { app: app };
}

impl DockerCompose<'_> {
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

    fs::read_dir(dir.clone()).expect(&format!("Failed to read {:?}", dir)).for_each(|e| {
      let path = e.unwrap().path();

      if path.is_file() && path != new_file {
        fs::remove_file(path.clone()).expect(&format!("Failed to remove {:?}", path));
      }
    });

    let dhcp = self.create_override(Box::new(new_file));
    let dns = self.app.config.dns();

    dns.update_config(self.app, dhcp.dns_config());
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
      .expect(&format!("Failed to execute shasum -a 256 {:?}", file.as_os_str()))
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
          // [NOTE]
          //   Removing the expose part.
          //   It's necessary to consider for embedded environment variable.
          //   https://docs.docker.com/compose/environment-variables/env-file/
          //
          // TODO: To support longsyntax format
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
            panic!("Failed to read {:?} : unsupport `ports: [...]` format", path)
          }
        }
        writeln!(writer, "{}", l).expect(&write_error_message);
      }
    }

    writer.flush().expect(&write_error_message);

    return Some(Box::new(path));
  }

  fn create_override(&self, file: Box<PathBuf>) -> dhcp::Dhcp {
    let yaml = docker_compose::load(file);
    let dns = self.app.config.dns();
    let mut dhcp = dns.new_dhcp_for(self.app);

    let orig_services = yaml.services.unwrap_or(BTreeMap::new());
    // let orig_networks = yaml.networks.unwrap_or(BTreeMap::new());

    let mut services: BTreeMap<String, docker_compose::Service> = BTreeMap::new();
    for (name, service) in orig_services.iter() {
      let orig_name = service.container_name.as_ref().unwrap_or(name);

      let mut nw: BTreeMap<String, docker_compose::Network> = BTreeMap::new();
      nw.insert(
        dns.name.clone(),
        docker_compose::Network {
          external: None,
          ipv4_address: Some(dhcp.assign(orig_name)),
          aliases: None,
        },
      );
      // TODO: To support the case when networks is configured in original
      nw.insert(
        "default".to_string(),
        docker_compose::Network {
          external: None,
          ipv4_address: None,
          aliases: Some(vec![orig_name.to_string()]),
        },
      );

      let s = docker_compose::Service {
        container_name: Some(format!("{}-{}", self.app.name.clone(), orig_name)),
        build: None,
        volumes: None,
        ports: None,
        networks: Some(docker_compose::ServiceNetworkable::Map(nw)),
        dns: Some(vec![dns.addr(), dns::global_addr()]),
        tty: None,
        stdin_open: None,
      };

      services.insert(name.clone(), s);
    }

    let mut networks: BTreeMap<String, docker_compose::Network> = BTreeMap::new();

    networks.insert(
      dns.name.clone(),
      docker_compose::Network {
        external: Some(true),
        ipv4_address: None,
        aliases: None,
      },
    );

    let yaml = docker_compose::Yaml {
      version: None,
      services: Some(services),
      networks: Some(networks),
    };

    yaml.save(Box::new(self.dist_root().join("override.yml")));

    return dhcp;
  }
}
