use std::process;

pub fn on() -> bool {
  let uname = process::Command::new("uname").arg("-a").output().expect("failed to execute uname").stdout.iter().map(|&x| x as char).collect::<String>();
  let parts = uname.split(" ").collect::<Vec<&str>>().into_iter().take(2);

  return parts.eq(vec!["Linux", "lima-hills"]);
}

pub fn should() -> bool {
  let uname = process::Command::new("uname").arg("-a").output().expect("failed to execute uname").stdout.iter().map(|&x| x as char).collect::<String>();
  let parts = uname.split(" ").collect::<Vec<&str>>().into_iter().take(1);

  return !parts.eq(vec!["Linux"]);
}

pub fn login() {
  // Lima::new().start()
}

pub fn shutdown() {}

pub fn destroy() {}
