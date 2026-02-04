use clap::Parser;
use smc_cli::{
    command::{CliArgs, Commands},
    func,
};

fn main() {
    let cli = CliArgs::parse();
    match cli.command {
        Commands::List => {
            if let Err(e) = func::list() {
                eprintln!("Error: {e}");
            }
        }
        Commands::Read { key } => {
            if let Err(e) = func::read(&key) {
                eprintln!("Error: {e}");
            }
        }
        Commands::Write { key, value } => {
            if let Err(e) = func::write(&key, &value) {
                eprintln!("Error: {e}");
            }
        }
    }
}
