use crossterm::style::Color;
use tree_sitter::Parser;
use tree_sitter_highlight::{
    Error, Highlight, HighlightConfiguration, HighlightEvent, Highlighter,
};

#[derive(PartialOrd, PartialEq, Eq, Ord, Clone)]
// Order is Priotity when multiple groups are active
enum HighlightGroup {
    Flags,
    Anchors,
    Quantifiers,
    CharacterClass,
    Operator,
    Escape,
    Group,
}

macro_rules! group_data {
    ($group:ident, $group_name:expr, $color:expr, [$($token:expr),*]) => {
        HighlightGroupData {
            group: HighlightGroup::$group,
            group_name: $group_name,
            color: $color,
            query: {
                const MATCHES: &str = concat!(
                    "[ ",
                    $(concat!("(", $token, ") "),)*
                    "]"
                );
                Some(MATCHES)
            }
        }
    };
    ($group:ident, $group_name:expr, $color:expr) => {
        HighlightGroupData {
            group: HighlightGroup::$group,
            group_name: $group_name,
            color: $color,
            query: None,
        }
    };
}

struct HighlightGroupData {
    group: HighlightGroup,
    group_name: &'static str,
    color: Color,
    query: Option<&'static str>,
}

impl HighlightGroup {
    fn all() -> &'static [HighlightGroupData] {
        &[
            // if no token used, then the group definition comes from tree_sitter_regex::HIGHLIGHTS_QUERY
            group_data!(Flags, "flags", Color::Blue, ["flags", "inline_flags_group"]),
            group_data!(Anchors, "anchors", Color::Magenta, [
                "start_assertion",
                "end_assertion",
                "boundary_assertion",
                "non_boundary_assertion"
            ]),
            group_data!(Quantifiers, "quantifiers", Color::DarkMagenta, [
                "one_or_more",
                "optional",
                "zero_or_more",
                "count_quantifier"
            ]),
            group_data!(CharacterClass, "character_class", Color::DarkBlue, [
                "character_class_escape",
                "character_class"
            ]),
            group_data!(Operator, "operator", Color::DarkYellow),
            group_data!(Escape, "escape", Color::Black),
            group_data!(Group, "property", Color::Black),
        ]
    }

    fn data(&self) -> &'static HighlightGroupData {
        Self::all().iter().find(|x| &x.group == self).unwrap()
    }

    fn color(&self) -> Color {
        self.data().color
    }

    fn group_name(&self) -> &'static str {
        self.data().group_name
    }

    fn group_names() -> Vec<&'static str> {
        Self::all().iter().map(|x| x.group_name).collect()
    }

    fn query(&self) -> Option<String> {
        let data = self.data();
        data.query.map(|q| format!("{} @{}", q, self.group_name()))
    }
}

fn custom_queries() -> String {
    HighlightGroup::all()
        .iter()
        .flat_map(|x| x.group.query())
        .collect::<Vec<String>>()
        .join("\n")
}

fn highlight_configuration() -> Result<HighlightConfiguration, Error> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_regex::LANGUAGE.into())
        .map_err(|_| Error::InvalidLanguage)?;

    let highlights_query = [tree_sitter_regex::HIGHLIGHTS_QUERY, &custom_queries()].join("\n");

    let mut highlight_configuration = HighlightConfiguration::new(
        tree_sitter_regex::LANGUAGE.into(),
        "regex",
        &highlights_query,
        "",
        "",
    )
    .map_err(|_| Error::Unknown)?;

    let hightlight_groups = HighlightGroup::group_names();

    highlight_configuration.configure(&hightlight_groups);
    Ok(highlight_configuration)
}

#[derive(Default)]
pub struct HighlightEventWrapper {
    iter: std::vec::IntoIter<Result<HighlightEvent, Error>>,
    pos: usize,
    limit: usize,
    stack: Vec<HighlightGroup>,
}

impl HighlightEventWrapper {
    pub fn new(re: &[u8]) -> Result<Self, Error> {
        let mut highlighter = Highlighter::new();
        let config = highlight_configuration()?;
        let highlights = highlighter.highlight(&config, re, None, |_| None)?;
        Ok(HighlightEventWrapper {
            iter: highlights.collect::<Vec<_>>().into_iter(),
            ..Default::default()
        })
    }
}

impl Iterator for HighlightEventWrapper {
    type Item = Color;
    fn next(&mut self) -> Option<Color> {
        if self.pos < self.limit {
            self.pos += 1;
            return self
                .stack
                .iter()
                .min()
                .map(|group| group.color())
                .or(Some(Color::Reset));
        }

        if let Some(Ok(event)) = self.iter.next() {
            match event {
                HighlightEvent::HighlightStart(Highlight(num)) => {
                    self.stack.push(HighlightGroup::all()[num].group.clone());
                }
                HighlightEvent::Source { start: _, end } => {
                    self.limit = end;
                }
                HighlightEvent::HighlightEnd => {
                    self.stack.pop();
                }
            }
            self.next()
        } else {
            None
        }
    }
}
