mod parser;

use amethyst::{
    assets::{Asset, Format, Handle},
    ecs::HashMapStorage,
    error::Error,
};

/// A sequence of passages, associated with a speaker.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PassageGroup {
    pub speaker: String,
    /// Blocks of text to show, one by one.
    pub passages: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dialogue {
    pub passage_groups: Vec<PassageGroup>,
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
        Ok(parser::parse(&String::from_utf8(bytes)?)?)
    }
}