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
    /// List all smc keys and values
    List,
    /// Read the smc value of the key
    Read {
        #[arg(help = "smc key name")]
        key: String,
    },

    /// Write smc value
    Write {
        #[arg(help = "smc key name")]
        key: String,
        #[arg(help = "smc value in hex, for 0x031000, write 031000")]
        value: String,
    },
}
