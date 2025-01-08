use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Color,
    terminal::DisableLineWrap,
};
use input::Input;
use persist::Session;
use regex::Cache as RegexCache;
use render::Render;

mod input;
pub mod persist;
mod regex;
mod render;

const LINES_BETWEEN: u16 = 3;

const RE_TITLE: &str = "REGULAR EXPRESSION: ";
const HAY_TITLE: &str = "TEST STRING       : ";

const LEFT_PADDING: u16 = max(RE_TITLE.len(), HAY_TITLE.len()) as u16;

const LAYER_COLORS: [Color; 6] = [
    Color::Grey, // marks the main match itself
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
];

const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

struct Group {
    start: usize,
    end: usize,
    layer: usize,
}

enum Field {
    RegexQuery,
    TestString,
}

struct Change {
    content: bool,
    cursor: bool,
}

impl Change {
    pub fn new() -> Self {
        Self {
            content: false,
            cursor: false,
        }
    }

    pub fn content(mut self) -> Self {
        self.content = true;
        self
    }

    pub fn cursor(mut self) -> Self {
        self.cursor = true;
        self
    }
}

impl Default for Change {
    fn default() -> Self {
        Self::new()
    }
}

pub struct App<W: io::Write> {
    session: Session,
    render: Render<W>,
    field: Field,
    regex_cache: RegexCache,
    exit: bool,
}

impl<W: io::Write> App<W> {
    pub fn new(w: W, session: Session) -> Self {
        Self {
            session,
            render: Render::new(w),
            field: Field::RegexQuery,
            regex_cache: RegexCache::new(),
            exit: false,
        }
    }

    pub fn run(mut self) -> io::Result<Session> {
        let mut change = Change::new().cursor().content();
        self.render.queue(DisableLineWrap)?;

        while !self.exit {
            if change.content {
                self.draw()?;
            }
            if change.cursor {
                let (col, row) = self.pos();
                self.render.move_to(col, row)?;
            }
            if change.content || change.cursor {
                self.render.flush()?;
            }

            change = self.handle_events()?;
        }

        // clear the screen after exiting
        self.render.move_to(0, 0)?;
        self.render.clear()?;
        self.render.flush()?;

        Ok(self.session)
    }

    fn draw(&mut self) -> io::Result<()> {
        self.render.clear()?;

        self.render.at(Color::Reset, RE_TITLE, 0, 0)?;
        self.render.at(Color::Reset, HAY_TITLE, 0, LINES_BETWEEN)?;

        self.render
            .draw_regex_query(&self.session.regex_query.string, LEFT_PADDING, 0)?;
        self.draw_hay(LEFT_PADDING, LINES_BETWEEN)
    }

    fn handle_events(&mut self) -> io::Result<Change> {
        let change = match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => Change::new(),
        };
        Ok(change)
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Change {
        match key_event.code {
            KeyCode::Char(ch) => {
                if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
                    match ch {
                        'h' => self.current_field().move_cursor_left(),
                        'j' | 'k' | 'n' | 'p' => self.switch(),
                        'l' => self.current_field().move_cursor_right(),
                        _ => Change::new(),
                    }
                } else {
                    self.current_field().insert(ch)
                }
            }
            KeyCode::Backspace => self.current_field().delete_char(),
            KeyCode::Left => {
                if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
                    self.current_field().move_cursor_start()
                } else {
                    self.current_field().move_cursor_left()
                }
            }
            KeyCode::Right => {
                if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
                    self.current_field().move_cursor_end()
                } else {
                    self.current_field().move_cursor_right()
                }
            }
            KeyCode::Tab | KeyCode::Up | KeyCode::Down => self.switch(),
            KeyCode::Esc => self.exit(),
            _ => Change::new(),
        }
    }

    fn exit(&mut self) -> Change {
        self.exit = true;
        Change::new()
    }

    fn current_field(&mut self) -> &mut Input {
        match self.field {
            Field::RegexQuery => &mut self.session.regex_query,
            Field::TestString => &mut self.session.test_string,
        }
    }

    fn switch(&mut self) -> Change {
        self.field = match self.field {
            Field::RegexQuery => Field::TestString,
            Field::TestString => Field::RegexQuery,
        };
        Change::new().cursor()
    }

    fn draw_hay(&mut self, col: u16, row: u16) -> io::Result<()> {
        match self.regex_cache.get_or_init(
            &self.session.regex_query.string,
            &self.session.test_string.string,
        ) {
            Ok(matches) => {
                self.render
                    .draw_regex_hay(&self.session.test_string.string, matches, col, row)
            }
            Err(err) => self.render.draw_error(&err.to_string(), col, row),
        }
    }

    fn pos(&self) -> (u16, u16) {
        match self.field {
            Field::RegexQuery => (LEFT_PADDING + self.session.regex_query.cursor as u16, 0),
            Field::TestString => (
                LEFT_PADDING + self.session.test_string.cursor as u16,
                LINES_BETWEEN,
            ),
        }
    }
}
