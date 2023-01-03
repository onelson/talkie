// A lot of this was copied from `talkie/assets/mod.rs` but leaves behind all
// the amethyst-specific asset-loader support.

use anyhow::Result;
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
    pub fn from_slice(bytes: &[u8]) -> Result<Dialogue> {
        let mut dialogue: Dialogue = toml::from_slice(bytes)?;
        for passage in dialogue
            .passage_groups
            .iter_mut()
            .flat_map(|x| x.passages.iter_mut())
        {
            *passage = reflow_text(passage);
        }

        Ok(dialogue)
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

/// Given some amount of time, use the rate to determine how much of the time
/// went unused and how many glyphs should now be revealed.
pub(crate) fn calc_glyphs_to_reveal(delta_secs: f32, glyphs_per_sec: f32) -> (usize, f32) {
    let reveal_how_many = (delta_secs * glyphs_per_sec).trunc();
    let remainder = delta_secs - (reveal_how_many / glyphs_per_sec);
    (reveal_how_many as usize, remainder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    /// If the delta is not big enough to reveal at least one glyph, then the
    /// remainder should be the entire delta.
    #[test]
    fn test_delta_carries_over() {
        let (count, remainder) = calc_glyphs_to_reveal(1.0, 0.5);
        assert_eq!(0, count);
        assert_approx_eq!(1.0, remainder);
    }

    #[test]
    fn test_delta_zero_when_glyph_revealed() {
        let (count, remainder) = calc_glyphs_to_reveal(2.0, 0.5);
        assert_eq!(1, count);
        assert_approx_eq!(0.0, remainder);
    }

    #[test]
    fn test_delta_remainder_when_glyph_revealed() {
        let (count, remainder) = calc_glyphs_to_reveal(2.2, 0.5);
        assert_eq!(1, count);
        assert_approx_eq!(0.2, remainder);
    }

    #[test]
    fn test_multi_glyph_remainder() {
        let (count, remainder) = calc_glyphs_to_reveal(5.2, 2.0);
        assert_eq!(10, count);
        assert_approx_eq!(0.2, remainder);
    }

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
