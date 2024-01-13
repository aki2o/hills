use crate::config::Config;
use crate::docker_compose;
use std::collections::BTreeMap;
use std::fs;
use std::net::Ipv4Addr;

pub struct Dns<'a> {
  config: &'a Config,
}

pub fn new<'a>(config: &'a Config) -> Dns<'a> {
  return Dns { config: config };
}

impl Dns<'_> {
  pub fn ensure_docker_compose(&self) {
    let dir = Config::global_dir_path().join("dns");

    if !dir.exists() {
      fs::create_dir_all(dir.clone()).expect(&format!("Failed to create {:?}", dir));
    }

    let network_name = "hills";
    let ipv4_address = Ipv4Addr::new(172, 31, 0, 2);

    let mut services: BTreeMap<String, docker_compose::Service> = BTreeMap::new();
    let mut networks: BTreeMap<String, docker_compose::Network> = BTreeMap::new();
    let mut service_networks: BTreeMap<String, docker_compose::Network> = BTreeMap::new();

    service_networks.insert(
      network_name.to_string(),
      docker_compose::Network {
        external: None,
        ipv4_address: Some(ipv4_address),
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
        volumes: Some(vec!["./unbound_conf.d:/etc/unbound/unbound.conf.d".to_string()]),
        ports: Some(vec!["53:53".to_string(), "53:53/udp".to_string()]),
        networks: Some(docker_compose::ServiceNetworkable::Map(service_networks)),
        dns: None,
        tty: None,
        stdin_open: None,
      },
    );

    networks.insert(
      network_name.to_string(),
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
