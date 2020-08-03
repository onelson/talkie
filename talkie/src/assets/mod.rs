use amethyst::{
    assets::{Asset, Format, Handle},
    ecs::HashMapStorage,
    error::Error,
};
use serde::Deserialize;

/// Sections that include one or more choices will present a menu to the player
/// once all the passage text has been shown. The last passage will be displayed
/// as the prompt for the choices.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Choice {
    /// The text to display in the menu.
    pub label: String,
    /// When  specified, this is used as a section (matched by id) to jump to.
    /// If no goto is listed, the choice simply advances to the next section.
    pub goto: Option<String>,
}

/// A sequence of passages, associated with a speaker.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct PassageGroup {
    /// This optional id is how `Choice`s find the passage group to jump to when
    /// a value for `goto` is set.
    pub id: Option<String>,
    pub speaker: Option<String>,
    /// Blocks of text to show, one by one.
    pub passages: Vec<String>,
    pub choices: Option<Vec<Choice>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Dialogue {
    #[serde(rename = "section")]
    pub passage_groups: Vec<PassageGroup>,
}

impl Dialogue {
    pub fn from_slice(bytes: &[u8]) -> Result<Dialogue, Error> {
        let mut dialogue: Dialogue = toml::from_slice(&bytes)?;
        for passage in dialogue
            .passage_groups
            .iter_mut()
            .flat_map(|x| x.passages.iter_mut())
        {
            *passage = reflow_text(&passage);
        }

        Ok(dialogue)
    }
}

/// A handle to a `Dialogue` asset.
pub type DialogueHandle = Handle<Dialogue>;

impl Asset for Dialogue {
    const NAME: &'static str = "talkie::dialogue::Dialogue";
    type Data = Self;
    type HandleStorage = HashMapStorage<DialogueHandle>;
}

/// Format for loading from `.dialogue` files.
#[derive(Clone, Copy, Debug, Default)]
pub struct DialogueFormat;

impl Format<Dialogue> for DialogueFormat {
    fn name(&self) -> &'static str {
        "DialogueFormat"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<Dialogue, Error> {
        Ok(Dialogue::from_slice(&bytes)?)
    }
}

/// Elide consecutive lines of text.
///
/// Single line breaks are joined and double line breaks are converted to single
/// line breaks.
///
/// A quirk of how this works is you'll find an extra space at the end of each
/// line. In practice this might not matter to you, but in the case that it
/// does... too bad!
fn reflow_text(input: &str) -> String {
    input.lines().fold(String::new(), |mut acc, s| {
        let s = s.trim();
        let sep = if s.is_empty() { "\n" } else { " " };
        acc.push_str(s);
        acc.push_str(sep);
        acc
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflow_single_line() {
        assert_eq!("abc", reflow_text("abc").trim());
    }

    #[test]
    fn test_reflow_lines_no_blanks() {
        assert_eq!(
            "abc def ",
            reflow_text(
                "
                abc
                def
                "
                .trim()
            )
        );
    }

    #[test]
    fn test_reflow_lines_with_blanks() {
        assert_eq!(
            "\
abc def 
ghi jkl \
",
            reflow_text(
                "
        abc
        def
        
        ghi
        jkl
        "
                .trim()
            )
        );
    }
}
