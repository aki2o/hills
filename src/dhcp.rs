use ipnet::{IpAdd, Ipv4Net};
use std::net::Ipv4Addr;

pub struct Dhcp {
  domain: String,
  subnet: Ipv4Net,
}

pub fn new(domain: String, subnet: Ipv4Net) -> Dhcp {
  return Dhcp { domain: domain, subnet: subnet };
}

impl Dhcp {
  pub fn assign(&self, service: &str) -> Ipv4Addr {
    return self.subnet.addr().saturating_add(3);
  }
}
