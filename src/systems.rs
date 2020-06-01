//! ECS Systems.
//!
//! In an ECS, *systems* drive change to the "world" (a collection of entities
//! and resources) in each tick of the game loop.

use crate::action_tracker::ActionTracker;
use crate::billboard::BillboardData;
use crate::dialogue::Dialogue;
use amethyst::{
    assets::AssetStorage,
    derive::SystemDesc,
    ecs::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
    ui::UiText,
};
use log::debug;

/// Updates the display of the billboard text.
#[derive(SystemDesc)]
pub struct BillboardDisplaySystem;

impl<'s> System<'s> for BillboardDisplaySystem {
    type SystemData = (
        WriteStorage<'s, BillboardData>,
        WriteStorage<'s, UiText>,
        Read<'s, AssetStorage<Dialogue>>,
        ReadStorage<'s, ActionTracker>,
    );

    fn run(&mut self, (mut billboard, mut ui_text, dialogues, action_tracker): Self::SystemData) {
        // TODO write out one glyph per <unit of time> instead of per tick.
        for (text, billboard, tracker) in (&mut ui_text, &mut billboard, &action_tracker).join() {
            if tracker.press_begin {
                billboard.paused = !billboard.paused;
            }

            if billboard.paused {
                return;
            }
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

#[derive(SystemDesc)]
pub struct ActionTrackerSystem;

impl<'s> System<'s> for ActionTrackerSystem {
    type SystemData = (
        WriteStorage<'s, ActionTracker>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (mut tracker, input): Self::SystemData) {
        for tracker in (&mut tracker).join() {
            let is_down = input.action_is_down(tracker.action).unwrap_or(false);
            match (is_down, tracker.pressed) {
                (true, false) => {
                    tracker.press_begin = true;
                    tracker.pressed = true;
                    tracker.press_end = false;
                    debug!("{}=down", tracker.action);
                }
                (true, true) => {
                    tracker.press_begin = false;
                    tracker.pressed = true;
                    tracker.press_end = false;
                    debug!("{}=pressed", tracker.action);
                }
                (false, true) => {
                    tracker.press_begin = false;
                    tracker.pressed = false;
                    tracker.press_end = true;
                    debug!("{}=up", tracker.action);
                }
                (false, false) => {
                    tracker.press_begin = false;
                    tracker.pressed = false;
                    tracker.press_end = false;
                }
            }
        }
    }
}
