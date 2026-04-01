use clap::{Parser, Subcommand};

use crate::commands;
use crate::commands::tag::TagAction;

#[derive(Debug, Parser)]
#[command(
    name = "wid",
    version,
    about = "Track what you're doing in a global markdown log",
    after_help = "\
Examples:
  wid
    Show the current log.
  wid add add examples to md-edit help
    Add a pending item without changing focus.
  wid now support note editing in rm -i
    Add a new active item and focus it immediately.
  wid done -i
    Toggle done state interactively.
  wid --json
    Print the log as JSON for agents or scripts.
  wid done --id 8f3c2d1a6b4e
    Mark a specific item as done from a transient id.
  wid note -i
    Choose an item interactively and append a note to it.
  wid tag add --id 8f3c2d1a6b4e @wid
    Add tags to a specific item by transient id.

Run 'wid <command> --help' for details."
)]
pub struct Cli {
    #[arg(long = "json", help = "Print the log as JSON")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(
        about = "Add a pending item without changing the current focus",
        after_help = "\
Examples:
  wid add add examples to md-edit help
    Add a pending item to the backlog.
  echo '--json output follow-up' | wid add
    Add a pending item from standard input.
  wid add
    Prompt for one line of input and add it as pending."
    )]
    Add {
        #[arg(
            help = "The item text to add. If omitted, wid prompts for one line of input.",
            allow_hyphen_values = true
        )]
        text: Vec<String>,
    },
    #[command(
        about = "Move done items into the archive log",
        after_help = "\
Examples:
  wid archive
    Ask for confirmation, then move all done items from log.md into archive.md.
  wid archive --yes
    Skip confirmation and archive done items immediately."
    )]
    Archive {
        #[arg(long = "yes", help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(
        about = "Mark an item as done",
        after_help = "\
Examples:
  wid done
    Mark the active item, or the latest open item, as done.
  wid done -i
    Toggle done state for multiple items interactively.
  wid done --id 8f3c2d1a6b4e
    Mark a specific item as done from a transient id."
    )]
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
    #[command(
        about = "Edit an existing item summary",
        after_help = "\
Examples:
  wid edit
    Edit the active item, or the latest item.
  wid edit --id 8f3c2d1a6b4e rename this task
    Edit a specific item or note by transient id.
  wid edit -i
    Choose an item or note from the inline picker and edit it."
    )]
    Edit {
        #[arg(
            short = 'i',
            long = "interactive",
            help = "Choose an item interactively",
            conflicts_with = "id"
        )]
        interactive: bool,
        #[arg(long = "id", help = "Edit a specific item or note by transient id")]
        id: Option<String>,
        #[arg(
            help = "The updated text. If omitted, wid prompts for one line of input.",
            allow_hyphen_values = true
        )]
        text: Vec<String>,
    },
    #[command(
        about = "Focus an existing item",
        after_help = "\
Examples:
  wid focus
    Focus the latest unfinished item.
  wid focus -i
    Choose which item to focus from the inline picker."
    )]
    Focus {
        #[arg(
            short = 'i',
            long = "interactive",
            help = "Choose an item interactively"
        )]
        interactive: bool,
    },
    #[command(
        about = "Remove an item",
        after_help = "\
Examples:
  wid rm -i
    Remove an item or note from the inline picker.
  wid rm --id note_4a1d9c2e7f55
    Remove a specific item or note by transient id."
    )]
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
    #[command(
        about = "Add a new active item and focus it immediately",
        after_help = "\
Examples:
  wid now support note editing in rm -i
    Add a new active item and focus it immediately.
  echo '--id support for agent workflows' | wid now
    Add a new active item from standard input.
  wid now
    Prompt for one line of input and make it active."
    )]
    Now {
        #[arg(
            help = "The item text to start now. If omitted, wid prompts for one line of input.",
            allow_hyphen_values = true
        )]
        text: Vec<String>,
    },
    #[command(
        about = "Add a note under the current or latest open item",
        after_help = "\
Examples:
  wid note align delete confirmation copy for notes
    Add a note to the active item, or the latest open item.
  wid note --id 8f3c2d1a6b4e waiting for CI to finish
    Add a note to a specific item by transient id.
  wid note -i
    Choose which item should receive the note.
  echo '--json shape' | wid note --id 8f3c2d1a6b4e
    Add a note from standard input to a specific item.
  wid note
    Prompt for one line of input and add it as a note."
    )]
    Note {
        #[arg(
            short = 'i',
            long = "interactive",
            help = "Choose an item interactively",
            conflicts_with = "id"
        )]
        interactive: bool,
        #[arg(
            help = "The note text to add. If omitted, wid prompts for one line of input.",
            allow_hyphen_values = true
        )]
        text: Vec<String>,
        #[arg(long = "id", help = "Add a note to a specific item by transient id")]
        id: Option<String>,
    },
    #[command(
        about = "Open the log file in $EDITOR",
        after_help = "\
Examples:
  wid open
    Open log.md in $EDITOR.
  wid open --archive
    Open archive.md in $EDITOR."
    )]
    Open {
        #[arg(long = "archive", help = "Open archive.md instead of log.md")]
        archive: bool,
    },
    #[command(subcommand, about = "Add or remove tags on an item")]
    Tag(TagCommands),
}

#[derive(Debug, Subcommand)]
pub enum TagCommands {
    #[command(
        about = "Add tags to an item",
        after_help = "\
Examples:
  wid tag add --id 8f3c2d1a6b4e @wid @agent
    Add one or more @tags to a specific item."
    )]
    Add {
        #[arg(long = "id", help = "Target a specific item by transient id")]
        id: String,
        #[arg(help = "One or more tags to add, each starting with @")]
        tags: Vec<String>,
    },
    #[command(
        about = "Remove tags from an item",
        after_help = "\
Examples:
  wid tag rm --id 8f3c2d1a6b4e @agent
    Remove one or more @tags from a specific item."
    )]
    Rm {
        #[arg(long = "id", help = "Target a specific item by transient id")]
        id: String,
        #[arg(help = "One or more tags to remove, each starting with @")]
        tags: Vec<String>,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { text }) => commands::add::run(text),
        Some(Commands::Archive { yes }) => commands::archive::run(yes),
        Some(Commands::Done { interactive, id }) => commands::done::run(interactive, id),
        Some(Commands::Edit {
            interactive,
            id,
            text,
        }) => commands::edit::run(interactive, id, text),
        Some(Commands::Focus { interactive }) => commands::focus::run(interactive),
        Some(Commands::Rm { interactive, id }) => commands::rm::run(interactive, id),
        Some(Commands::Now { text }) => commands::now::run(text),
        Some(Commands::Note {
            text,
            id,
            interactive,
        }) => commands::note::run(text, id, interactive),
        Some(Commands::Open { archive }) => commands::open::run(archive),
        Some(Commands::Tag(TagCommands::Add { id, tags })) => {
            commands::tag::run(TagAction::Add, id, tags)
        }
        Some(Commands::Tag(TagCommands::Rm { id, tags })) => {
            commands::tag::run(TagAction::Rm, id, tags)
        }
        None => commands::show::run(cli.json),
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, Commands, TagCommands};
    use clap::Parser;

    #[test]
    fn add_accepts_hyphen_prefixed_text_without_double_dash() {
        let cli = Cli::try_parse_from(["wid", "add", "--id", "support", "for", "agents"])
            .expect("add should treat hyphen-prefixed values as text");

        match cli.command {
            Some(Commands::Add { text }) => {
                assert_eq!(text, vec!["--id", "support", "for", "agents"]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn now_accepts_hyphen_prefixed_text_without_double_dash() {
        let cli = Cli::try_parse_from(["wid", "now", "--id", "support", "for", "agents"])
            .expect("now should treat hyphen-prefixed values as text");

        match cli.command {
            Some(Commands::Now { text }) => {
                assert_eq!(text, vec!["--id", "support", "for", "agents"]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn note_accepts_hyphen_prefixed_text_after_target_id() {
        let cli = Cli::try_parse_from(["wid", "note", "--id", "entry_123", "--json", "shape"])
            .expect("note should allow hyphen-prefixed note text after --id");

        match cli.command {
            Some(Commands::Note { id, text, .. }) => {
                assert_eq!(id.as_deref(), Some("entry_123"));
                assert_eq!(text, vec!["--json", "shape"]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn edit_accepts_hyphen_prefixed_text_after_target_id() {
        let cli = Cli::try_parse_from(["wid", "edit", "--id", "entry_123", "--json", "shape"])
            .expect("edit should allow hyphen-prefixed text after --id");

        match cli.command {
            Some(Commands::Edit { id, text, .. }) => {
                assert_eq!(id.as_deref(), Some("entry_123"));
                assert_eq!(text, vec!["--json", "shape"]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn tag_add_parses_id_and_tags() {
        let cli = Cli::try_parse_from(["wid", "tag", "add", "--id", "entry_123", "@wid", "@agent"])
            .expect("tag add should parse id and tags");

        match cli.command {
            Some(Commands::Tag(TagCommands::Add { id, tags })) => {
                assert_eq!(id, "entry_123");
                assert_eq!(tags, vec!["@wid", "@agent"]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
