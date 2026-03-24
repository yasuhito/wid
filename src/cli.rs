use clap::{Parser, Subcommand};

use crate::commands;

#[derive(Debug, Parser)]
#[command(name = "wid")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Done,
    Now {
        text: Vec<String>,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Done) => commands::done::run(),
        Some(Commands::Now { text }) => commands::now::run(text),
        None => commands::show::run(),
    }
}
