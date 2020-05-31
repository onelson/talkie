//! ECS Systems.
//!
//! In an ECS, *systems* drive change to the "world" (a collection of entities
//! and resources) in each tick of the game loop.

use crate::billboard::BillboardData;
use crate::dialogue::Dialogue;
use amethyst::{
    assets::AssetStorage,
    derive::SystemDesc,
    ecs::{Join, Read, System, SystemData, WriteStorage},
    ui::UiText,
};

/// Updates the display of the billboard text.
#[derive(SystemDesc)]
pub struct BillboardDisplaySystem;

impl<'s> System<'s> for BillboardDisplaySystem {
    type SystemData = (
        WriteStorage<'s, UiText>,
        Read<'s, AssetStorage<Dialogue>>,
        WriteStorage<'s, BillboardData>,
    );

    fn run(&mut self, (mut ui_text, dialogues, mut billboard): Self::SystemData) {
        // TODO write out one glyph per <unit of time> instead of per tick.
        for (text, billboard) in (&mut ui_text, &mut billboard).join() {
            if let Some(dialogue) = dialogues.get(&billboard.dialogue) {
                // XXX: text/passages should not end up empty. If they are, it
                // there be a problem with the parser.
                debug_assert!(!dialogue.passages.is_empty());
                let entire_text = &dialogue.passages[billboard.passage];
                debug_assert!(!entire_text.is_empty());
                text.text = entire_text.chars().take(billboard.head).collect();

                let end_of_text = billboard.head == entire_text.len() - 1;
                let last_passage = billboard.passage == dialogue.passages.len() - 1;

                // Go back to the very start if we're at the end of the last
                // passage.
                // If we're at the end of any other passage, reset the head
                // but advance to the next passage.
                // Otherwise, reveal another glyph of the current passage.
                match (end_of_text, last_passage) {
                    (true, true) => {
                        billboard.head = 0;
                        billboard.passage = 0;
                    }
                    (true, false) => {
                        billboard.head = 0;
                        billboard.passage += 1;
                    }
                    (false, _) => {
                        billboard.head += 1;
                    }
                }
            }
        }
    }
}
