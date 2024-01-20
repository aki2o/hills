use std::path::PathBuf;
use std::process::Command;
// use tap::prelude::*;

pub struct Runner {
  project_name: String,
  project_directory: Box<PathBuf>,
  files: Vec<Box<PathBuf>>,
  parallel: Option<i32>,
}

pub fn new(name: String, dir: Box<PathBuf>) -> Runner {
  return Runner {
    project_name: name,
    project_directory: dir,
    files: vec![],
    parallel: None,
  };
}

impl Runner {
  pub fn add_file(&mut self, file: Box<PathBuf>) -> &mut Self {
    self.files.push(file);
    return self;
  }

  pub fn set_parallel(&mut self, parallel: i32) -> &mut Self {
    self.parallel = Some(parallel);
    return self;
  }

  pub fn ps(&self) {
    let mut cmd = self.make_command("ps");

    cmd.spawn().expect(&format!("Failed to run command : {:?}", cmd));
  }

  fn make_command(&self, name: &str) -> Box<Command> {
    let mut cmd = Command::new("docker");

    cmd.arg("compose").arg(name);
    cmd.arg("-p").arg(&self.project_name);
    cmd.arg("--project-directory").arg(self.project_directory.as_os_str());

    self.files.iter().for_each(|f| {
      cmd.arg("-f").arg(f.as_os_str());
    });

    if let Some(parallel) = self.parallel {
      cmd.arg("--parallel").arg(parallel.to_string());
    }

    return Box::new(cmd);
  }
}
