use clap::{Args, Parser, Subcommand};
use hills::application;
use hills::config;
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

  /// List applications
  List,

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
      config::create(root);
    }
    Actions::List => {
      let c = config::load_from(root);

      for name in c.application_names() {
        println!("{}", name);
      }
    }
    Actions::New(args) => {
      let c = config::load_from(root);

      if !c.app_root().exists() {
        fs::create_dir_all(*c.app_root()).unwrap();
      }

      application::create(&c, &args.name);
    }
    Actions::Alias(args) => {
      let mut c = config::load_from(root);

      c.set_alias(&args.original, &args.alias);

      if let Some(name) = args.original {
        application::find_by(&c, &name).update(true);
      }
    }
    Actions::Update(args) => {
      let c = config::load_from(root);
      let app = application::find_by(&c, &args.name);

      app.update(false);

      c.dns().ensure_docker_compose();
    }
  }
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert()
}
