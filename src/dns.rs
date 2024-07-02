use crate::application::Application;
use crate::config;
use crate::dhcp;
use crate::docker_compose;
use ipnet::{IpAdd, Ipv4Net};
use std::collections::BTreeMap;
use std::fs;
use std::net::Ipv4Addr;
use std::path::PathBuf;

pub struct Dns {
  path: String,
  name: String,
  domain: String,
  subnet: Ipv4Net,
  root: Ipv4Addr,
}

pub fn new(path: String, name: String, domain: String, subnet: Ipv4Net, root: Ipv4Addr) -> Dns {
  return Dns {
    path: path,
    name: name,
    domain: domain,
    subnet: subnet,
    root: root,
  };
}

impl Dns {
  pub fn domain(&self) -> &str {
    return self.domain.as_str();
  }

  pub fn addr(&self) -> Ipv4Addr {
    return self.subnet.addr().saturating_add(2);
  }

  pub fn root_addr(&self) -> Ipv4Addr {
    return self.root.clone();
  }

  pub fn new_dhcp_for(&self, app: &Application) -> dhcp::Dhcp {
    return dhcp::new(app.domain(), self.find_or_create_subnet_for(app));
  }

  pub fn setup(&self) {
    if !self.docker_compose_path().exists() {
      self.create_docker_compose();
    }

    if !self.base_config_path().exists() {
      self.create_base_config();
    }
  }

  pub fn update_config(&self, app: &Application, value: String) {
    let file = self.dist_root().join(format!("{}.conf", app.name()));

    config::create_file(Box::new(file), value);
  }

  pub fn clear(&self) {
    let dir = self.root();

    if dir.exists() {
      fs::remove_dir_all(*dir.clone()).expect(&format!("Failed to remove {:?}", dir));
    }
  }

  fn create_docker_compose(&self) {
    let mut services: BTreeMap<String, docker_compose::Service> = BTreeMap::new();
    let mut networks: BTreeMap<String, docker_compose::Network> = BTreeMap::new();
    let mut service_networks: BTreeMap<String, docker_compose::Network> = BTreeMap::new();

    service_networks.insert(
      self.name.clone(),
      docker_compose::Network {
        external: None,
        ipv4_address: Some(self.addr().clone()),
        aliases: None,
      },
    );

    services.insert(
      "dns".to_string(),
      docker_compose::Service {
        container_name: None,
        build: Some(docker_compose::ServiceBuild {
          context: Some(".".to_string()),
          dockerfile: Some("unbound.Dockerfile".to_string()),
        }),
        // TODO: Making the dist path be programmatic
        volumes: Some(vec!["../.dist/unbound.conf.d:/etc/unbound/unbound.conf.d".to_string()]),
        ports: Some(vec!["53:53".to_string(), "53:53/udp".to_string()]),
        networks: Some(docker_compose::ServiceNetworkable::Map(service_networks)),
        dns: None,
        tty: None,
        stdin_open: None,
      },
    );

    networks.insert(
      self.name.clone(),
      docker_compose::Network {
        external: Some(true),
        ipv4_address: None,
        aliases: None,
      },
    );

    let yaml = docker_compose::Yaml {
      version: Some("3.8".to_string()),
      services: Some(services),
      networks: Some(networks),
    };

    yaml.save(self.docker_compose_path());
  }

  fn create_base_config(&self) {
    let s = r#"server:
  verbosity: 3
  use-syslog: no
  logfile: ""

  interface: 0.0.0.0
  interface: ::0
  access-control: 0.0.0.0/0 allow

  local-zone: "local." transparent
"#
    .to_string();

    config::create_file(self.base_config_path(), s);
  }

  fn find_or_create_subnet_for(&self, app: &Application) -> Ipv4Net {
    let file = self.root().join("subnets.toml");
    let key = app.name();
    let mut subnets: BTreeMap<String, Ipv4Net> = BTreeMap::new();

    if file.exists() {
      let s = fs::read_to_string(file.clone()).expect(&format!("Failed to read {:?}", file));
      subnets = toml::from_str(&s).expect(&format!("Failed to load config from {:?}", file));
    }

    return match subnets.get(&key) {
      Some(v) => v.clone(),
      None => {
        let subnet = self
          .subnet
          .subnets(24)
          .unwrap()
          .find(|s| !self.subnet.addr().eq(&s.addr()) && !subnets.values().any(|v| v.eq(s)))
          .expect("Not found avaiable subnet! Please run stop --all"); // TODO

        subnets.insert(key, subnet.clone());
        config::create_file(Box::new(file), toml::to_string(&subnets).unwrap());

        subnet
      }
    };
  }

  fn docker_compose_path(&self) -> Box<PathBuf> {
    return Box::new(self.root().join("docker-compose.yml"));
  }

  fn base_config_path(&self) -> Box<PathBuf> {
    return Box::new(self.dist_root().join("base.conf"));
  }

  fn root(&self) -> Box<PathBuf> {
    let dir = config::current().root().join(self.path);

    if !dir.exists() {
      fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
    }

    return Box::new(dir);
  }

  fn dist_root(&self) -> Box<PathBuf> {
    let dir = config::current().dist_root().join("unbound.conf.d");

    if !dir.exists() {
      fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
    }

    return Box::new(dir);
  }
}
