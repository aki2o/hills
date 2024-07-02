const FILES: BTreeMap<&str, &str> = BTreeMap::from([
  ("01_rootable.sh", "sudo cp ~/.ssh/authorized_keys /root/.ssh/"),
  ("10_tz.sh", "timedatectl set-timezone Asia/Tokyo"),
  (
    "10_ip_forwarding.sh",
    r#"
bash -c "sed -i 's/^#net\.ipv4\.ip_forward/net.ipv4.ip_forward=1/' /etc/sysctl.conf"
bash -c "sed -i 's/^#net\.ipv6\.conf\.all\.forwarding/net.ipv6.conf.all.forwarding=1/' /etc/sysctl.conf"
"#,
  ),
  // https://github.com/moby/moby/issues/22635
  (
    "30_dns_server.sh",
    r#"
bash -c "sed -i 's/^#DNS=.*$/DNS=8.8.8.8/' /etc/systemd/resolved.conf"
"#,
  ),
  // https://qiita.com/shora_kujira16/items/31d09b373809a5a44ae5
  (
    "30_dns_stub_listener.sh",
    r#"
bash -c "sed -i 's/^#DNSStubListener=.*$/DNSStubListener=no/' /etc/systemd/resolved.conf"
"#,
  ),
  (
    "50_apt.sh",
    r#"
apt update
apt -y install --no-install-recommends build-essential ruby mysql-client uidmap dbus-user-session net-tools
rm -rf /var/lib/apt/lists/* /var/cache/apt/*
"#,
  ),
  (
    "70_docker.sh",
    r#"
curl -fsSL https://get.docker.com | sh

curl -L "https://github.com/docker/compose/releases/download/v2.10.0/docker-compose-$(uname -s)-$(uname -m)" -o /tmp/docker-compose
chmod +x /tmp/docker-compose
mv /tmp/docker-compose /usr/local/bin/docker-compose
"#,
  ),
  (
    "90_restart_service.sh",
    r#"
sysctl --system

systemctl restart multipathd.service

ln -sf ../run/systemd/resolve/resolv.conf /etc/resolv.conf
systemctl restart systemd-resolved
"#,
  ),
  // https://qiita.com/yn-misaki/items/c850a07f7858437e4d26
  // https://qiita.com/tachibanayu24/items/951b358fffeb0378ff53
  (
    "/etc/sysctl.d/60-inotify-limit.conf",
    r#"
fs.inotify.max_user_watches=524288
fs.inotify.max_user_instances=256
"#,
  ),
  // https://sleeplessbeastie.eu/2021/01/06/how-to-fix-multipath-daemon-error-about-missing-path-when-using-virtualbox/
  (
    "/etc/multipath.conf",
    r#"
defaults {
    user_friendly_names yes
}

blacklist {
    device {
        vendor "VBOX"
        product "HARDDISK"
    }
}
"#,
  ),
  // http://itemy.net/?p=539
  (
    "/root/.profile",
    r#"
[ "$BASH" -a -f ~/.bashrc ] && . ~/.bashrc
tty -s && mesg n
"#,
  ),
]);

pub fn all() -> Vec<Box<PathBuf>> {
  return FILES.keys().map(|k| Box::new(PathBuf::from(k)) as Box<PathBuf>).collect();
}

pub fn script_of(path: Box<PathBuf>) -> String {
  return script_with(FILES[path.to_str().unwrap()]);
}

fn script_with(script: &str) -> String {
  return vec!["#!/bin/bash", "set -eux -o pipefail", script].join("\n");
}
