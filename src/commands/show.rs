use std::io::{self, Write};

use anyhow::Context;

use crate::log::{format::format_log, store::load_log};

pub fn run() -> anyhow::Result<()> {
    let document = load_log()?;
    let output = format_log(&document);

    io::stdout()
        .write_all(output.as_bytes())
        .context("failed to write log output")?;

    Ok(())
}
