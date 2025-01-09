use std::{cmp, fmt::Display, io};

use crossterm::{
    Command,
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use crate::{Group, LAYER_COLORS, highlight::HighlightEventWrapper};

pub struct Render<W: io::Write>(W);

impl<W: io::Write> Render<W> {
    pub fn new(w: W) -> Self {
        Self(w)
    }

    #[inline]
    pub fn queue(&mut self, command: impl Command) -> io::Result<()> {
        queue!(self.0, command)
    }

    #[inline]
    pub fn clear(&mut self) -> io::Result<()> {
        queue!(self.0, Clear(ClearType::All))
    }

    #[inline]
    pub fn draw<T>(&mut self, color: Color, text: T) -> io::Result<()>
    where
        T: Display,
    {
        queue!(self.0, SetForegroundColor(color), Print(text))
    }

    #[inline]
    pub fn move_to(&mut self, col: u16, row: u16) -> io::Result<()> {
        queue!(self.0, MoveTo(col, row))
    }

    pub fn at<T>(&mut self, color: Color, text: T, col: u16, row: u16) -> io::Result<()>
    where
        T: Display,
    {
        self.move_to(col, row)?;
        self.draw(color, text)
    }

    #[inline]
    pub fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }

    pub fn draw_regex_query(&mut self, s: &str, col: u16, row: u16) -> io::Result<()> {
        self.move_to(col, row)?;
        let mut layer = 0;
        let mut syntax_highlighting = HighlightEventWrapper::new(s.as_bytes()).unwrap_or_default();
        for ch in s.chars() {
            let syntax_color = syntax_highlighting
                .by_ref()
                .take(ch.len_utf8())
                .last()
                .unwrap_or(Color::Reset);

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
                _ => syntax_color,
            };
            self.draw(color, ch)?;
        }

        Ok(())
    }

    pub fn draw_regex_hay(
        &mut self,
        s: &str,
        matches: &Vec<Vec<(usize, usize)>>,
        col: u16,
        row: u16,
    ) -> io::Result<()> {
        self.at(Color::Reset, s, col, row)?;

        for captures in matches {
            let (max_layer, infos) = self.draw_regex_match(s, captures, col, row)?;
            self.draw_regex_groups(&infos, col, row, max_layer)?;
        }

        Ok(())
    }

    fn draw_regex_match(
        &mut self,
        s: &str,
        captures: &Vec<(usize, usize)>,
        col: u16,
        row: u16,
    ) -> io::Result<(usize, Vec<Group>)> {
        let mut layers = Vec::new();
        let mut infos = Vec::new();
        let mut max_layer = 0;

        for &(start, end) in captures {
            while layers.last().is_some_and(|l| *l <= start) {
                layers.pop();
            }
            layers.push(end);

            let color = LAYER_COLORS[layers.len() - 1];

            self.at(color, &s[start..end], col + start as u16, row)?;

            let layer = layers.len() - 1;
            infos.push(Group { start, end, layer });

            max_layer = cmp::max(max_layer, layer);
        }

        Ok((max_layer, infos))
    }

    fn draw_regex_groups(
        &mut self,
        infos: &[Group],
        col: u16,
        row: u16,
        max_layer: usize,
    ) -> Result<(), io::Error> {
        for &Group { start, end, layer } in infos {
            let color = LAYER_COLORS[layer];
            let (start, end, layer) = (start as u16, end as u16, layer as u16);
            let max_layer = max_layer as u16;

            for idx in start..end.saturating_sub(1) {
                self.at(color, '~', col + idx, row + layer + 1)?;
            }
            self.at(color, '|', col + end.saturating_sub(1), row + layer + 1)?;

            for line in layer + 1..=max_layer + 1 {
                self.at(color, '|', col + start, row + line)?;
            }
            self.at(color, layer, col + start, row + max_layer + 2)?;
        }

        Ok(())
    }

    pub fn draw_error(&mut self, s: &str, col: u16, row: u16) -> io::Result<()> {
        self.at(Color::DarkRed, "ERROR", col, row)?;
        for (i, line) in s.lines().enumerate() {
            self.at(Color::Reset, line, col, row + 1 + i as u16)?
        }
        Ok(())
    }
}
