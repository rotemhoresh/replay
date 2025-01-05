use crate::Change;

#[derive(Default)]
pub struct Input {
    pub string: String,
    pub cursor: usize,
}

impl Input {
    pub fn insert(&mut self, ch: char) -> Change {
        let index = self.byte_index();
        self.string.insert(index, ch);
        self.move_cursor_right();
        Change::new().cursor().content()
    }

    pub fn delete_char(&mut self) -> Change {
        if self.cursor > 0 {
            let before = self.string.chars().take(self.cursor - 1);
            let after = self.string.chars().skip(self.cursor);

            self.string = before.chain(after).collect();
            self.move_cursor_left();

            Change::new().content().cursor()
        } else {
            Change::new()
        }
    }

    pub fn move_cursor_end(&mut self) -> Change {
        self.cursor = self.string.len();
        Change::new().cursor()
    }

    pub fn move_cursor_start(&mut self) -> Change {
        self.cursor = 0;
        Change::new().cursor()
    }

    pub fn move_cursor_left(&mut self) -> Change {
        let cursor_moved_left = self.cursor.saturating_sub(1);
        self.cursor = self.clamp_cursor(cursor_moved_left);
        Change::new().cursor()
    }

    pub fn move_cursor_right(&mut self) -> Change {
        let cursor_moved_right = self.cursor.saturating_add(1);
        self.cursor = self.clamp_cursor(cursor_moved_right);
        Change::new().cursor()
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
