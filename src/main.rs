pub mod cli;
pub mod commands;
pub mod log;

fn main() -> anyhow::Result<()> {
    cli::run()
}
