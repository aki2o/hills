use crate::application::Application;
use crate::docker_compose::command;
use std::path::PathBuf;

pub struct Runner {
  name: String,
  root: Box<PathBuf>,
  files: Vec<Box<PathBuf>>,
}

pub fn new(app: &Application, files: Vec<Box<PathBuf>>) -> Runner {
  return Runner {
    name: app.name().to_string(),
    root: app.root(),
    files: files,
  };
}

impl Runner {
  pub fn ps(&self) {
    let mut cmd = self.make_command().ps();

    cmd.spawn().expect(&format!("Failed to run command : {:?}", cmd));
  }

  fn make_command(&self) -> Box<command::Command> {
    let mut cmd = command::new(self.name.clone(), self.root.clone());

    self.files.iter().for_each(|f| {
      cmd.add_file(f.clone());
    });

    return Box::new(cmd);
  }
}
