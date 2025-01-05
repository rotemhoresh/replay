use std::{fmt::Display, io};

use cache::RegexCache;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType, DisableLineWrap},
};
use input::Input;

mod cache;
mod input;

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

enum Type {
    Re,
    Hay,
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

pub struct App {
    typ: Type,
    regex: RegexCache,
    re: Input,
    hay: Input,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            typ: Type::Re,
            regex: RegexCache::new(),
            re: Input::default(),
            hay: Input::default(),
            exit: false,
        }
    }

    pub fn run<W>(&mut self, w: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        let mut change = Change::new().cursor().content();
        queue!(w, DisableLineWrap)?;

        while !self.exit {
            if change.content {
                self.draw(w)?;
            }
            if change.cursor {
                let (col, row) = self.pos();
                queue!(w, MoveTo(col, row))?;
            }
            if change.content || change.cursor {
                w.flush()?;
            }

            change = self.handle_events()?;
        }

        // clear the screen after exiting
        execute!(w, MoveTo(0, 0), Clear(ClearType::All))
    }

    fn draw<W>(&mut self, w: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        queue!(w, Clear(ClearType::All))?;

        print_at(w, Color::Reset, RE_TITLE, 0, 0)?;
        print_at(w, Color::Reset, HAY_TITLE, 0, LINES_BETWEEN)?;

        self.draw_re(w, LEFT_PADDING, 0)?;
        self.draw_hay(w, LEFT_PADDING, LINES_BETWEEN)
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
                        'h' => self.current_mut().move_cursor_left(),
                        'j' | 'k' | 'n' | 'p' => self.switch(),
                        'l' => self.current_mut().move_cursor_right(),
                        _ => Change::new(),
                    }
                } else {
                    self.current_mut().insert(ch)
                }
            }
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
            _ => Change::new(),
        }
    }

    fn exit(&mut self) -> Change {
        self.exit = true;
        Change::new()
    }

    fn current_mut(&mut self) -> &mut Input {
        match self.typ {
            Type::Re => &mut self.re,
            Type::Hay => &mut self.hay,
        }
    }

    fn switch(&mut self) -> Change {
        self.typ = match self.typ {
            Type::Re => Type::Hay,
            Type::Hay => Type::Re,
        };
        Change::new().cursor()
    }

    pub fn draw_re<W>(&self, w: &mut W, col: u16, row: u16) -> io::Result<()>
    where
        W: io::Write,
    {
        queue!(w, MoveTo(col, row))?;

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
                _ => Color::Reset,
            };
            print(w, color, ch)?;
        }

        Ok(())
    }

    fn draw_hay<W>(&mut self, w: &mut W, col: u16, row: u16) -> io::Result<()>
    where
        W: io::Write,
    {
        let caps = match self.regex.get_or_init(&self.re.string, &self.hay.string) {
            Ok(caps) => caps,
            Err(err) => return draw_error(w, &err.to_string(), col, row),
        };

        print_at(w, Color::Reset, &self.hay.string, col, row)?;

        for cap in caps {
            let (max_layer, infos) = draw_match(w, &self.hay.string, cap, col, row)?;
            draw_groups(w, &infos, col, row, max_layer)?;
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

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn draw_error<W>(w: &mut W, err: &str, col: u16, row: u16) -> io::Result<()>
where
    W: io::Write,
{
    print_at(w, Color::DarkRed, "ERROR:", col, row)?;
    for (i, line) in err.lines().enumerate() {
        print_at(w, Color::Reset, line, col, row + 1 + i as u16)?
    }
    Ok(())
}

fn draw_match<W>(
    w: &mut W,
    hay: &str,
    cap: &Vec<(usize, usize)>,
    col: u16,
    row: u16,
) -> io::Result<(usize, Vec<Group>)>
where
    W: io::Write,
{
    let mut layers = Vec::new();
    let mut infos = Vec::new();
    let mut max_layer = 0;

    for &(start, end) in cap {
        while layers.last().is_some_and(|l| *l <= start) {
            layers.pop();
        }
        layers.push(end);

        let color = LAYER_COLORS[layers.len() - 1];

        print_at(w, color, &hay[start..end], col + start as u16, row)?;

        let layer = layers.len() - 1;
        infos.push(Group { start, end, layer });

        if layer > max_layer {
            max_layer = layers.len() - 1;
        }
    }

    Ok((max_layer, infos))
}

fn draw_groups<W>(
    w: &mut W,
    infos: &[Group],
    col: u16,
    row: u16,
    max_layer: usize,
) -> Result<(), io::Error>
where
    W: io::Write,
{
    for &Group { start, end, layer } in infos {
        let color = LAYER_COLORS[layer];
        let (start, end, layer) = (start as u16, end as u16, layer as u16);
        let max_layer = max_layer as u16;

        for idx in start..end.saturating_sub(1) {
            print_at(w, color, '~', col + idx, row + layer + 1)?;
        }
        print_at(w, color, '|', col + end.saturating_sub(1), row + layer + 1)?;

        for line in layer + 1..=max_layer + 1 {
            print_at(w, color, '|', col + start, row + line)?;
        }
        print_at(w, color, layer, col + start, row + max_layer + 2)?;
    }

    Ok(())
}

fn print<W, T>(w: &mut W, bg: Color, text: T) -> io::Result<()>
where
    W: io::Write,
    T: Display,
{
    queue!(
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
    queue!(w, MoveTo(col, row))?;
    print(w, bg, text)
}
