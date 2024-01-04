use clap::{Args, Parser, Subcommand};
use hills::config::Config;
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

fn main() {
    let cli = Cli::parse();
    let root = Path::new(&cli.context);
    println!("root {:?}", root.to_str());

    match cli.action {
        Actions::Init => {
            Config::create(root);
        }
        Actions::New(args) => {
            Config::load_from(root);
        }
        Actions::Alias(args) => {
            println!("Alias: {:?}", args);
        }
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
