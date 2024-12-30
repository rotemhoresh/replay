use std::io::{self, Write, stdout};

use crossterm::{
    cursor,
    event::{Event, KeyCode, read},
    execute, style,
    terminal::{self, ClearType},
};
use regex::Regex;

const LINES_BETWEEN: u16 = 3;

const RE_TITLE: &str = "REGULAR EXPRESSION: ";
const HAY_TITLE: &str = "TEST STRING: ";

const LEFT_PADDING: u16 = max(RE_TITLE.len(), HAY_TITLE.len()) as u16;

const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

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

    pub fn print_result<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        let re = if let Ok(re) = Regex::new(&self.re) {
            re
        } else {
            return execute!(
                w,
                style::SetBackgroundColor(style::Color::DarkRed),
                style::Print("ERROR"),
                style::SetBackgroundColor(style::Color::Reset),
            );
        };
        let caps = re.captures_iter(&self.hay);

        let mut last_cap = 0;

        for cap in caps {
            let c = cap.get(0).unwrap(); // this cannot return None
            let start = c.start();
            let end = c.end();
            let mut last_mat = start;

            execute!(w, style::Print(&self.hay[last_cap..start]))?;

            for mat in cap.iter().flatten().skip(1) {
                let start = mat.start();
                let end = mat.end();

                execute!(
                    w,
                    style::SetBackgroundColor(style::Color::DarkGrey),
                    style::Print(&self.hay[last_mat..start]),
                    style::SetBackgroundColor(style::Color::DarkGreen),
                    style::Print(&self.hay[start..end]),
                    style::SetBackgroundColor(style::Color::Reset)
                )?;

                last_mat = end;
            }

            if last_mat < end {
                execute!(
                    w,
                    style::SetBackgroundColor(style::Color::DarkGrey),
                    style::Print(&self.hay[last_mat..end]),
                    style::SetBackgroundColor(style::Color::Reset)
                )?;
            }

            last_cap = end;
        }

        if last_cap < self.hay.len() {
            execute!(w, style::Print(&self.hay[last_cap..]))?;
        }

        // match matches {
        //     Ok(matches) => {
        //         let mut last = 0;
        //         for  in matches {
        //             execute!(
        //                 w,
        //                 style::Print(&self.hay[last..start]),
        //                 style::SetBackgroundColor(style::Color::DarkGreen),
        //                 style::Print(&self.hay[start..end]),
        //                 style::SetBackgroundColor(style::Color::Reset)
        //             )?;
        //             last = end;
        //         }
        //         if last != self.hay.len() {
        //             execute!(w, style::Print(&self.hay[last..]))?;
        //         }
        //     }
        //     Err(_) => execute!(w, style::Print("error"))?,
        // };
        Ok(())
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
            style::Print(RE_TITLE),
            cursor::MoveTo(LEFT_PADDING, 0),
            style::Print(input.re()),
            cursor::MoveTo(0, LINES_BETWEEN),
            style::Print(HAY_TITLE),
            cursor::MoveTo(LEFT_PADDING, LINES_BETWEEN),
        )?;
        input.print_result(&mut stdout)?;
        execute!(stdout, cursor::MoveTo(col + LEFT_PADDING, row))?;
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
