use std::{fmt::Display, io};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use regex::Regex;

const LINES_BETWEEN: u16 = 3;

const RE_TITLE: &str = "REGULAR EXPRESSION: ";
const HAY_TITLE: &str = "TEST STRING: ";

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

enum Type {
    Re,
    Hay,
}

#[derive(Default)]
struct Input {
    string: String,
    cursor: usize,
}

impl Input {
    pub fn insert(&mut self, ch: char) {
        let index = self.byte_index();
        self.string.insert(index, ch);
        self.move_cursor_right();
    }

    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            let before = self.string.chars().take(self.cursor - 1);
            let after = self.string.chars().skip(self.cursor);

            self.string = before.chain(after).collect();
            self.move_cursor_left();
        }
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor = self.string.len();
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor.saturating_sub(1);
        self.cursor = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor.saturating_add(1);
        self.cursor = self.clamp_cursor(cursor_moved_right);
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.string.chars().count())
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.string
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor)
            .unwrap_or(self.string.len())
    }
}

struct App {
    typ: Type,
    re: Input,
    hay: Input,
    exit: bool,
}

impl App {
    pub fn run<W>(&mut self, w: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        while !self.exit {
            self.draw(w)?;
            self.handle_events()?;
        }

        // clear the screen after exiting
        execute!(w, MoveTo(0, 0), Clear(ClearType::All))?;
        w.flush()
    }

    fn draw<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        execute!(w, Clear(ClearType::All))?;

        print_at(w, Color::Reset, RE_TITLE, 0, 0)?;
        print_at(w, Color::Reset, HAY_TITLE, 0, LINES_BETWEEN)?;
        print_layers_color(w, LEFT_PADDING, LINES_BETWEEN * 3)?;

        self.draw_re(w, LEFT_PADDING, 0)?;
        self.draw_hay(w, LEFT_PADDING, LINES_BETWEEN)?;

        let (col, row) = self.pos();
        execute!(w, MoveTo(col, row))?;

        w.flush()
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(ch) => self.current_mut().insert(ch),
            KeyCode::Backspace => self.current_mut().delete_char(),
            KeyCode::Left => {
                if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
                    self.current_mut().move_cursor_start()
                } else {
                    self.current_mut().move_cursor_left()
                }
            }
            KeyCode::Right => {
                if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
                    self.current_mut().move_cursor_end()
                } else {
                    self.current_mut().move_cursor_right()
                }
            }
            KeyCode::Tab | KeyCode::Up | KeyCode::Down => self.switch(),
            KeyCode::Esc => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn current_mut(&mut self) -> &mut Input {
        match self.typ {
            Type::Re => &mut self.re,
            Type::Hay => &mut self.hay,
        }
    }

    pub fn new() -> Self {
        Self {
            typ: Type::Re,
            re: Input::default(),
            hay: Input::default(),
            exit: false,
        }
    }

    fn switch(&mut self) {
        self.typ = match self.typ {
            Type::Re => Type::Hay,
            Type::Hay => Type::Re,
        };
    }

    pub fn draw_re<W>(&self, w: &mut W, col: u16, row: u16) -> io::Result<()>
    where
        W: io::Write,
    {
        execute!(w, MoveTo(col, row))?;

        let mut layer = 0;

        for ch in self.re.string.chars() {
            let color = match ch {
                '(' => {
                    layer += 1;
                    LAYER_COLORS[layer]
                }
                ')' => {
                    let color = LAYER_COLORS[layer];
                    layer = layer.saturating_sub(1);
                    color
                }
                _ => Color::DarkGrey,
            };
            print(w, color, ch)?;
        }

        Ok(())
    }

    fn draw_hay<W>(&self, w: &mut W, col: u16, row: u16) -> io::Result<()>
    where
        W: io::Write,
    {
        let re = match Regex::new(&self.re.string) {
            Ok(re) => re,
            Err(err) => {
                print_at(w, Color::DarkRed, "ERROR:", col, row)?;
                for (i, line) in err.to_string().lines().enumerate() {
                    print_at(w, Color::Reset, line, col, row + 1 + i as u16)?;
                }
                return Ok(());
            }
        };
        let caps = re.captures_iter(&self.hay.string);

        print_at(w, Color::Reset, &self.hay.string, col, row)?;

        for cap in caps {
            let mut layers = Vec::new();
            let mut infos = Vec::new();

            for mat in cap.iter().flatten() {
                while layers.last().is_some_and(|l| *l <= mat.start()) {
                    layers.pop();
                }
                layers.push(mat.end());

                let color = LAYER_COLORS[layers.len() - 1];

                print_at(
                    w,
                    color,
                    &self.hay.string[mat.start()..mat.end()],
                    col + mat.start() as u16,
                    row,
                )?;

                infos.push((mat.start(), layers.len() - 1));
            }

            for (i, (idx, layer)) in infos.iter().enumerate() {
                let color = LAYER_COLORS[*layer];
                let col = col + *idx as u16;

                for line in 1..=infos.len() - i + 1 {
                    print_at(w, color, '|', col, row + line as u16)?;
                }

                print_at(w, color, layer, col, row + (infos.len() - i) as u16 + 1)?;
            }
        }

        Ok(())
    }

    fn pos(&self) -> (u16, u16) {
        match self.typ {
            Type::Re => (LEFT_PADDING + self.re.cursor as u16, 0),
            Type::Hay => (LEFT_PADDING + self.hay.cursor as u16, LINES_BETWEEN),
        }
    }
}

fn print<W, T>(w: &mut W, bg: Color, text: T) -> io::Result<()>
where
    W: io::Write,
    T: Display,
{
    execute!(
        w,
        SetForegroundColor(bg),
        Print(text),
        SetForegroundColor(Color::Reset)
    )
}

fn print_at<W, T>(w: &mut W, bg: Color, text: T, col: u16, row: u16) -> io::Result<()>
where
    W: io::Write,
    T: Display,
{
    execute!(w, MoveTo(col, row))?;
    print(w, bg, text)
}

fn print_layers_color<W>(w: &mut W, col: u16, row: u16) -> io::Result<()>
where
    W: io::Write,
{
    execute!(w, MoveTo(col, row))?;
    for color in LAYER_COLORS {
        print(w, color, "  ")?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let app_result = App::new().run(&mut io::stdout());
    terminal::disable_raw_mode()?;
    app_result
}
