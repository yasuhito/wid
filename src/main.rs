pub mod cli;
pub mod commands;
pub mod interactive;
pub mod log;

fn main() -> anyhow::Result<()> {
    cli::run()
}
