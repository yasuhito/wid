use clap::{Parser, Subcommand};

use crate::commands;

#[derive(Debug, Parser)]
#[command(
    name = "wid",
    about = "Track what you're doing in a global markdown log"
)]
pub struct Cli {
    #[arg(long = "json", help = "Print the log as JSON")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Add a pending item without changing the current focus")]
    Add {
        #[arg(help = "The item text to add. If omitted, wid prompts for one line of input.")]
        text: Vec<String>,
    },
    #[command(about = "Move done items into the archive log")]
    Archive,
    #[command(about = "Mark an item as done")]
    Done {
        #[arg(
            short = 'i',
            long = "interactive",
            help = "Choose an item interactively",
            conflicts_with = "id"
        )]
        interactive: bool,
        #[arg(long = "id", help = "Mark a specific item as done by transient id")]
        id: Option<String>,
    },
    #[command(about = "Edit an existing item summary")]
    Edit {
        #[arg(
            short = 'i',
            long = "interactive",
            help = "Choose an item interactively"
        )]
        interactive: bool,
    },
    #[command(about = "Focus an existing item")]
    Focus {
        #[arg(
            short = 'i',
            long = "interactive",
            help = "Choose an item interactively"
        )]
        interactive: bool,
    },
    #[command(about = "Remove an item")]
    Rm {
        #[arg(
            short = 'i',
            long = "interactive",
            help = "Choose an item interactively",
            conflicts_with = "id"
        )]
        interactive: bool,
        #[arg(long = "id", help = "Remove a specific item or note by transient id")]
        id: Option<String>,
    },
    #[command(about = "Add a new active item and focus it immediately")]
    Now {
        #[arg(help = "The item text to start now. If omitted, wid prompts for one line of input.")]
        text: Vec<String>,
    },
    #[command(about = "Add a note under the current or latest open item")]
    Note {
        #[arg(help = "The note text to add. If omitted, wid prompts for one line of input.")]
        text: Vec<String>,
        #[arg(long = "id", help = "Add a note to a specific item by transient id")]
        id: Option<String>,
    },
    #[command(about = "Open the log file in $EDITOR")]
    Open {
        #[arg(long = "archive", help = "Open archive.md instead of log.md")]
        archive: bool,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { text }) => commands::add::run(text),
        Some(Commands::Archive) => commands::archive::run(),
        Some(Commands::Done { interactive, id }) => commands::done::run(interactive, id),
        Some(Commands::Edit { interactive }) => commands::edit::run(interactive),
        Some(Commands::Focus { interactive }) => commands::focus::run(interactive),
        Some(Commands::Rm { interactive, id }) => commands::rm::run(interactive, id),
        Some(Commands::Now { text }) => commands::now::run(text),
        Some(Commands::Note { text, id }) => commands::note::run(text, id),
        Some(Commands::Open { archive }) => commands::open::run(archive),
        None => commands::show::run(cli.json),
    }
}
