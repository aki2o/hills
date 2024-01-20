use ipnet::{IpAdd, Ipv4Net};
use std::collections::BTreeMap;
use std::net::Ipv4Addr;

pub struct Dhcp {
  domain: String,
  subnet: Ipv4Net,
  services: BTreeMap<String, Ipv4Addr>,
}

pub fn new(domain: String, subnet: Ipv4Net) -> Dhcp {
  return Dhcp {
    domain: domain,
    subnet: subnet,
    services: BTreeMap::new(),
  };
}

impl Dhcp {
  pub fn assign(&mut self, service: &str) -> Ipv4Addr {
    let index = self.services.keys().count() + 1;
    let addr = self.subnet.addr().saturating_add(index as u32);

    self.services.insert(service.to_string(), addr.clone());

    return addr;
  }

  pub fn dns_config(&self) -> String {
    return self
      .services
      .keys()
      .map(|s| format!("local-data: \"{}.{}.local. A {}\"\n", s, self.domain, self.services.get(s).unwrap()))
      .collect::<Vec<String>>()
      .join("");
  }
}
