//! ECS Systems.
//!
//! In an ECS, *systems* drive change to the "world" (a collection of entities
//! and resources) in each tick of the game loop.

use crate::components::ActionTracker;
use bevy::{
    input::{keyboard::KeyCode, Input},
    prelude::*,
};

fn action_tracker(query: Query<Res<Input<KeyCode>>, With<ActionTracker>>) {
    for tracker in query.iter() {
        let input = (); // FIXME
        tracker.update(&input);
    }
}
