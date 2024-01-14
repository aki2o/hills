use clap::{Args, Parser, Subcommand};
use hills::application::Application;
use hills::config::Config;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
  #[command(subcommand)]
  action: Actions,

  /// Set root directory
  #[arg(short, long)]
  #[arg(default_value = ".")]
  context: String,
}

#[derive(Subcommand)]
enum Actions {
  /// Initialize the configuration file.
  Init,

  /// Create a new application configuration file.
  New(NewArgs),

  /// Create an alias for the application.
  Alias(AliasArgs),

  /// Update the application.
  Update(UpdateArgs),
}

#[derive(Args, Debug)]
struct NewArgs {
  name: String,
}

#[derive(Args, Debug)]
struct AliasArgs {
  original: Option<String>,
  alias: Option<String>,
}

#[derive(Args, Debug)]
struct UpdateArgs {
  name: String,
}

fn main() {
  let cli = Cli::parse();
  let root = Path::new(&cli.context);

  match cli.action {
    Actions::Init => {
      Config::create(root);
    }
    Actions::New(args) => {
      let c = Config::load_from(root);

      if !c.app_root().exists() {
        fs::create_dir_all(*c.app_root()).unwrap();
      }

      Application::create(&c, &args.name);
    }
    Actions::Alias(args) => {
      println!("Alias: {:?}", args);
    }
    Actions::Update(args) => {
      let c = Config::load_from(root);
      let app = Application::find_by(&c, &args.name);

      app.docker_compose().sync();

      c.dns().ensure_docker_compose();
    }
  }
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert()
}
