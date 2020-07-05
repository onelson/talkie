//! ECS Systems.
//!
//! In an ECS, *systems* drive change to the "world" (a collection of entities
//! and resources) in each tick of the game loop.

use crate::assets::dialogue::Dialogue;
use crate::components::ActionTracker;
use crate::states::BillboardData;
use amethyst::{
    assets::AssetStorage,
    core::timing::Time,
    derive::SystemDesc,
    ecs::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    input::{InputHandler, StringBindings},
    ui::{UiFinder, UiText},
};
use log::debug;

/// Updates the display of the billboard text.
#[derive(SystemDesc)]
pub struct BillboardDisplaySystem {
    pub glyph_speed: Option<f32>,
}

/// A new glyph is revealed when this amount of time has passed.
const GLYPH_PERIOD_SECS: f32 = 0.2;

impl<'s> System<'s> for BillboardDisplaySystem {
    type SystemData = (
        Write<'s, BillboardData>,
        Read<'s, AssetStorage<Dialogue>>,
        ReadStorage<'s, ActionTracker>,
        UiFinder<'s>,
        WriteStorage<'s, UiText>,
        Read<'s, Time>,
    );

    fn run(
        &mut self,
        (mut billboard, dialogues, action_tracker, ui_finder, mut ui_text, time): Self::SystemData,
    ) {
        for tracker in (&action_tracker).join() {
            if tracker.press_begin {
                billboard.paused = !billboard.paused;
            }

            if billboard.paused {
                return;
            }

            if let Some(dialogue) = dialogues.get_by_id(billboard.dialogue_id) {
                let group = &dialogue.passage_groups[billboard.passage_group];

                if ui_finder
                    .find("speaker_name")
                    .and_then(|e| ui_text.get_mut(e))
                    .map(|t| t.text = format!("// {}", &group.speaker))
                    .is_none()
                {
                    // bail if we don't have a text display component to write to.
                    return;
                }

                let mut since = billboard.secs_since_last_reveal.unwrap_or_default();
                since += time.delta_seconds();

                let glyph_speed = self.glyph_speed.unwrap_or(GLYPH_PERIOD_SECS);
                let reveal_how_many = (since / glyph_speed).trunc() as usize;
                let remainder = since % glyph_speed;

                billboard.secs_since_last_reveal = Some(remainder);

                // XXX: text/passages should not end up empty. If they are, it
                // there be a problem with the parser.
                let entire_text = &group.passages[billboard.passage];

                if let Some(entity) = ui_finder.find("dialogue_text") {
                    if let Some(t) = ui_text.get_mut(entity) {
                        billboard.head += reveal_how_many; // Only advance if we can update the display
                        t.text = entire_text.chars().take(billboard.head).collect();
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
