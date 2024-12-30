use std::io::{self, Write, stdout};

use crossterm::{
    cursor,
    event::{Event, KeyCode, read},
    execute, style,
    terminal::{self, ClearType},
};
use regex::Regex;

const LINES_BETWEEN: u16 = 3;

enum Type {
    Re,
    Hay,
}

struct Input {
    typ: Type,
    re: String,
    hay: String,
}

impl Input {
    pub fn new() -> Self {
        Self {
            typ: Type::Re,
            re: String::new(),
            hay: String::new(),
        }
    }

    pub fn pop(&mut self) {
        match self.typ {
            Type::Re => self.re.pop(),
            Type::Hay => self.hay.pop(),
        };
    }

    pub fn push(&mut self, ch: char) {
        match self.typ {
            Type::Re => self.re.push(ch),
            Type::Hay => self.hay.push(ch),
        };
    }

    pub fn switch(&mut self) {
        self.typ = match self.typ {
            Type::Re => Type::Hay,
            Type::Hay => Type::Re,
        };
    }

    pub fn get(&self) -> &str {
        match self.typ {
            Type::Re => &self.re,
            Type::Hay => &self.hay,
        }
    }

    pub fn re(&self) -> &str {
        &self.re
    }

    pub fn hay(&self) -> &str {
        &self.hay
    }

    pub fn result(&self) -> String {
        Regex::new(&self.re)
            .map(|re| {
                let mut res = String::with_capacity(self.hay.len());
                let mut last = 0;

                for m in re.find_iter(&self.hay).map(|m| (m.start(), m.end())) {
                    res.push_str(&self.hay[last..m.0]);
                    res.push('[');
                    res.push_str(&self.hay[m.0..m.1]);
                    res.push(']');
                    last = m.1;
                }

                if last < self.hay.len() {
                    res.push_str(&self.hay[last..]);
                }

                res
            })
            .unwrap_or_else(|err| err.to_string())
    }

    pub fn pos(&self) -> (u16, u16) {
        match self.typ {
            Type::Re => (self.re.len() as u16, 0),
            Type::Hay => (self.hay.len() as u16, LINES_BETWEEN),
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    let (_, _) = terminal::size()?;

    terminal::enable_raw_mode()?;

    let mut input = Input::new();

    loop {
        let (col, row) = input.pos();
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            style::Print(input.re()),
            cursor::MoveTo(0, LINES_BETWEEN),
            style::Print(input.hay()),
            cursor::MoveTo(0, LINES_BETWEEN + 5),
            style::Print(input.result()),
            cursor::MoveTo(col, row),
        )?;
        stdout.flush()?;

        if let Event::Key(event) = read()? {
            match event.code {
                KeyCode::Esc => break,
                KeyCode::Backspace => input.pop(),
                KeyCode::Char(ch) => input.push(ch),
                KeyCode::Tab => input.switch(),
                _ => (),
            }
        }
    }

    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
    )?;

    terminal::disable_raw_mode()?;

    Ok(())
}
