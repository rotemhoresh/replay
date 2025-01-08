use std::{env, io};

use crossterm::terminal;
use replay::{
    App,
    persist::{DEFAULT_SESSION, Session},
};

#[inline]
fn session_name() -> String {
    env::args().nth(1).unwrap_or(DEFAULT_SESSION.to_owned())
}

fn main() -> anyhow::Result<()> {
    let session = Session::fetch(session_name()).unwrap();

    terminal::enable_raw_mode()?;
    let app_result = App::new(&mut io::stdout(), session).run();
    terminal::disable_raw_mode()?;

    let session = app_result?;

    if session.name != DEFAULT_SESSION {
        session.save()?;
    }
    Ok(())
}
