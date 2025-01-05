use std::io;

use crossterm::terminal;
use replay::App;

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let app_result = App::new().run(&mut io::stdout());
    terminal::disable_raw_mode()?;
    app_result
}
