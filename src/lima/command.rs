use std::path::PathBuf;
use std::process;

pub struct Command {
  name: String,
}

pub enum Status {
  NotCreated,
  PowerOff,
  Aborted,
  Running,
}

pub fn new(name: String) -> Command {
  return Command { name: name };
}

impl Command {
  pub fn status(&self) -> Status {
    let statuses = process::Command::new("limactl")
      .arg("list")
      .arg("-f")
      .arg("{{.Name}},{{.Status}}")
      .output()
      .expect("Failed to execute limactl list")
      .stdout
      .iter()
      .map(|&x| x as char)
      .collect::<String>()
      .trim()
      .split("\n")
      .collect::<Vec<&str>>();

    let status = statuses.find(|s| s.starts_with(&format!("{},", self.name)));

    return match status {
      Some(s) => {
        let parts = s.split(",").collect::<Vec<&str>>();

        match parts[1] {
          "not_created" => Status::NotCreated,
          "poweroff" => Status::PowerOff,
          "aborted" => Status::Aborted,
          "running" => Status::Running,
          _ => panic!("Unknown status : {}", parts[1]),
        }
      }
      None => Status::NotCreated,
    };
  }

  pub fn start(&self, file: Box<PathBuf>) -> bool {
    let output = process::Command::new("limactl")
      .arg("start")
      .arg("--name")
      .arg(&self.name)
      .arg(file.as_os_str())
      .output()
      .expect("Failed to execute limactl start");

    println!("{:?}", output.stdout);
    eprintln!("{:?}", output.stderr);

    return output.status.success();
  }
}
