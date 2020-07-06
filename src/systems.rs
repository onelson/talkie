//! ECS Systems.
//!
//! In an ECS, *systems* drive change to the "world" (a collection of entities
//! and resources) in each tick of the game loop.

use crate::components::ActionTracker;

use amethyst::{
    derive::SystemDesc,
    ecs::{Join, Read, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
};
use log::debug;

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
