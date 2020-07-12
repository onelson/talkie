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

#[derive(SystemDesc)]
pub struct ActionTrackerSystem;

impl<'s> System<'s> for ActionTrackerSystem {
    type SystemData = (
        WriteStorage<'s, ActionTracker>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (mut tracker, input): Self::SystemData) {
        for tracker in (&mut tracker).join() {
            tracker.update(&input);
        }
    }
}
