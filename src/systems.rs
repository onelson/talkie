//! ECS Systems.
//!
//! In an ECS, *systems* drive change to the "world" (a collection of entities
//! and resources) in each tick of the game loop.

use crate::assets::dialogue::Dialogue;
use crate::components::ActionTracker;
use crate::components::BillboardData;
use amethyst::{
    assets::AssetStorage,
    derive::SystemDesc,
    ecs::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
    ui::{UiFinder, UiText},
};
use log::debug;

/// Updates the display of the billboard text.
#[derive(SystemDesc)]
pub struct BillboardDisplaySystem;

impl<'s> System<'s> for BillboardDisplaySystem {
    type SystemData = (
        WriteStorage<'s, BillboardData>,
        Read<'s, AssetStorage<Dialogue>>,
        ReadStorage<'s, ActionTracker>,
        UiFinder<'s>,
        WriteStorage<'s, UiText>,
    );

    fn run(
        &mut self,
        (mut billboard, dialogues, action_tracker, ui_finder, mut ui_text): Self::SystemData,
    ) {
        // TODO write out one glyph per <unit of time> instead of per tick.
        for (billboard, tracker) in (&mut billboard, &action_tracker).join() {
            if tracker.press_begin {
                billboard.paused = !billboard.paused;
            }

            if billboard.paused {
                return;
            }

            if let Some(dialogue) = dialogues.get(&billboard.dialogue) {
                let group = &dialogue.passage_groups[billboard.passage_group];

                {
                    match ui_finder
                        .find("speaker_name")
                        .and_then(|e| ui_text.get_mut(e))
                    {
                        Some(t) => {
                            t.text = format!("// {}", &group.speaker);
                        }
                        // bail if we don't have a text display component to write to.
                        None => return,
                    }
                }

                // XXX: text/passages should not end up empty. If they are, it
                // there be a problem with the parser.
                let entire_text = &group.passages[billboard.passage];
                {
                    match ui_finder
                        .find("dialogue_text")
                        .and_then(|e| ui_text.get_mut(e))
                    {
                        Some(t) => {
                            t.text = entire_text.chars().take(billboard.head).collect();
                        }
                        // bail if we don't have a text display component to write to.
                        None => return,
                    }
                }

                let end_of_text = billboard.head == entire_text.len() - 1;
                let last_group = billboard.passage_group == dialogue.passage_groups.len() - 1;
                let last_passage = billboard.passage == group.passages.len() - 1;

                // Go back to the very start if we're at the end of the last
                // passage.
                // If we're at the end of any other passage, reset the head
                // but advance to the next passage.
                // Otherwise, reveal another glyph of the current passage.
                match (end_of_text, last_passage, last_group) {
                    (true, true, true) => {
                        billboard.head = 0;
                        billboard.passage_group = 0;
                        billboard.passage = 0;
                    }
                    (true, false, _) => {
                        billboard.head = 0;
                        billboard.passage += 1;
                    }
                    (true, true, false) => {
                        billboard.head = 0;
                        billboard.passage_group += 1;
                        billboard.passage = 0;
                    }
                    (false, _, _) => {
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
