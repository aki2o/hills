use clap::{Args, Parser, Subcommand};
use hills::application;
use hills::config;
use hills::vm;
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

  /// List applications
  List(ListArgs),

  /// Up the application.
  Up(UpArgs),

  /// Update the application.
  Update(UpdateArgs),

  /// Handle VM.
  Vm(VmArgs),
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
struct ListArgs {
  name: Option<String>,
}

#[derive(Args, Debug)]
struct UpArgs {
  name: String,
}

#[derive(Args, Debug)]
struct UpdateArgs {
  name: String,
}

#[derive(Args, Debug)]
struct VmArgs {
  action: String,
}

fn main() {
  let cli = Cli::parse();

  config::try_to_setup_from(Path::new(&cli.context));

  match cli.action {
    Actions::Init => {
      config::create(root);
    }
    Actions::New(args) => {
      if !config::current().app_root().exists() {
        fs::create_dir_all(*config::current().app_root()).unwrap();
      }

      application::create(&args.name);
    }
    Actions::Alias(args) => {
      config::set_alias(&args.original, &args.alias);

      // if let Some(name) = args.original {
      //   application::find_by(&c, &name).update(true);
      // }
    }
    Actions::List(args) => match &args.name {
      Some(name) => {
        application::find_by(name).print();
      }
      None => {
        for name in config::current().application_names() {
          println!("{}", name);
        }
      }
    },
    Actions::Up(args) => {
      if vm::should() && !vm::on() {
        panic!("You need to run on vm! Please do vm up");
      }
      let app = application::find_by(&args.name);

      app.update(false);
    }
    Actions::Update(args) => {
      let app = application::find_by(&args.name);

      app.update(false);
    }
    Actions::Vm(args) => match args.action.as_str() {
      "up" => {
        vm::login();
      }
      "down" => {
        vm::shutdown();
      }
      "clean" => {
        vm::destroy();
      }
      _ => {
        panic!("Invalid action : {}", args.action);
      }
    },
  }
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert()
}
