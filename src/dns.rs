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
  pub name: String,
  subnet: Ipv4Net,
}

pub fn new(name: String, subnet: Ipv4Net) -> Dns {
  return Dns { name: name, subnet: subnet };
}

pub fn root() -> Box<PathBuf> {
  return Box::new(config::global_dir_path().join("dns"));
}

pub fn global_addr() -> Ipv4Addr {
  return Ipv4Addr::new(8, 8, 8, 8);
}

impl Dns {
  pub fn addr(&self) -> Ipv4Addr {
    return self.subnet.addr().saturating_add(2);
  }

  pub fn new_dhcp_for(&self, app: &Application) -> dhcp::Dhcp {
    return dhcp::new(app.domain(), self.find_or_create_subnet_for(app));
  }

  fn find_or_create_subnet_for(&self, app: &Application) -> Ipv4Net {
    return *self.subnet.subnets(24).unwrap().collect::<Vec<Ipv4Net>>().first().unwrap();
  }

  pub fn ensure_docker_compose(&self) {
    let dir = root();

    if !dir.exists() {
      fs::create_dir_all(*dir.clone()).expect(&format!("Failed to create {:?}", dir));
    }

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
        volumes: Some(vec!["./unbound.conf.d:/etc/unbound/unbound.conf.d".to_string()]),
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

    yaml.save(Box::new(dir.join("docker-compose.yml")));
  }
}
