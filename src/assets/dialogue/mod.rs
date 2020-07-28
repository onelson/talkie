use crate::utils::reflow_text;
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
