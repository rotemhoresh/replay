use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::input::Input;

const INVALID_CHARS: [char; 3] = [' ', '/', '\\'];

pub enum SessionName {
    Scratch,
    Name(String),
}

#[derive(Debug)]
pub enum Error {
    InvalidName(char),
    InvalidFormat,
}

pub struct Session {
    pub name: SessionName,
    pub regex_query: Input,
    pub test_string: Input,
}

impl Session {
    pub fn fetch(name: String) -> Result<Self, Error> {
        validate_name(&name)?;

        let path = get_path(&name);

        let (regex_query, test_string) = parse_session(&path)?;

        Ok(Self {
            name: SessionName::Name(name),
            regex_query,
            test_string,
        })
    }

    pub fn scratch() -> Self {
        Self {
            name: SessionName::Scratch,
            regex_query: Input::default(),
            test_string: Input::default(),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        if let SessionName::Name(ref name) = self.name {
            let path = get_path(name);
            if let Some(p) = path.parent() {
                fs::create_dir_all(p)?;
            }
            if self.regex_query.string.is_empty() && self.test_string.string.is_empty() {
                // If the session if empty - don't save it, and make sure that there
                // is no file containing the previous snapshot of it.
                fs::remove_file(path)
            } else {
                fs::write(
                    &path,
                    format!(
                        "{}:{}\n{}:{}",
                        self.regex_query.cursor,
                        self.regex_query.string,
                        self.test_string.cursor,
                        self.test_string.string
                    ),
                )
            }
        } else {
            Ok(())
        }
    }
}

fn validate_name(name: &str) -> Result<(), Error> {
    if let Some(idx) = name.find(INVALID_CHARS) {
        Err(Error::InvalidName(name.chars().nth(idx).unwrap()))
    } else {
        Ok(())
    }
}

fn parse_session(path: &Path) -> Result<(Input, Input), Error> {
    if let Ok(s) = fs::read_to_string(path) {
        let lines: Vec<_> = s.split('\n').collect();
        if lines.len() != 2 {
            Err(Error::InvalidFormat)
        } else {
            let regex_query = parse_field(lines[0])?;
            let test_string = parse_field(lines[1])?;
            Ok((regex_query, test_string))
        }
    } else {
        // Create a blank session if the session file doesn't exist
        Ok((Input::default(), Input::default()))
    }
}

/// Creates a path to `~/.replay/persist/<name>`.
fn get_path(name: &str) -> PathBuf {
    let mut path = dirs::home_dir()
        .with_context(|| "failed to get home dir")
        .unwrap();
    path.push(".replay");
    path.push("persist");
    path.push(name);
    path
}

fn parse_field(s: &str) -> Result<Input, Error> {
    let (cursor, string) = s.split_once(':').ok_or(Error::InvalidFormat)?;
    let cursor = cursor.parse().map_err(|_| Error::InvalidFormat)?;
    Ok(Input {
        string: string.to_owned(),
        cursor,
    })
}
