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
    Add {
        text: Vec<String>,
    },
    Done {
        #[arg(short = 'i', long = "interactive")]
        interactive: bool,
    },
    Focus {
        #[arg(short = 'i', long = "interactive")]
        interactive: bool,
    },
    Rm {
        #[arg(short = 'i', long = "interactive")]
        interactive: bool,
    },
    Now {
        text: Vec<String>,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { text }) => commands::add::run(text),
        Some(Commands::Done { interactive }) => commands::done::run(interactive),
        Some(Commands::Focus { interactive }) => commands::focus::run(interactive),
        Some(Commands::Rm { interactive }) => commands::rm::run(interactive),
        Some(Commands::Now { text }) => commands::now::run(text),
        None => commands::show::run(),
    }
}
