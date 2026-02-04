use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "smc",
    version = "0.1.0",
    about = "Apple System Management Control (SMC) tool"
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all SMC keys and their values
    List,
    /// Read a single SMC key and display its value
    Read {
        #[arg(help = "Four-character SMC key name (e.g. TB0T, TCHP)")]
        key: String,
    },

    /// Write a value to a SMC key
    Write {
        #[arg(help = "Four-character SMC key name (e.g. TB0T, TCHP)")]
        key: String,
        #[arg(
            help = "Hexadecimal value to write (without `0x` prefix), for 0x031000, write 031000"
        )]
        value: String,
    },
}
