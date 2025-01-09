use std::{env, io};

use anyhow::Context;
use crossterm::terminal;
use replay::{App, persist::Session};

fn main() -> anyhow::Result<()> {
    let session = if let Some(name) = env::args().nth(1) {
        Session::fetch(name)?
    } else {
        Session::scratch()
    };

    terminal::enable_raw_mode()?;
    let session = App::new(&mut io::stdout(), session).run();
    terminal::disable_raw_mode()?;

    session?.save().with_context(|| "failed to save session")
}
