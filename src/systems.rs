//! ECS Systems.
//!
//! In an ECS, *systems* drive change to the "world" (a collection of entities
//! and resources) in each tick of the game loop.

use crate::billboard::BillboardData;
use amethyst::{
    derive::SystemDesc,
    ecs::{Join, System, SystemData, WriteStorage},
    ui::UiText,
};

/// Updates the display of the billboard text.
#[derive(SystemDesc)]
pub struct BillboardDisplaySystem;

impl<'s> System<'s> for BillboardDisplaySystem {
    type SystemData = (WriteStorage<'s, UiText>, WriteStorage<'s, BillboardData>);

    fn run(&mut self, (mut ui_text, mut billboard): Self::SystemData) {
        // TODO write out one glyph per <unit of time> instead of per tick.
        for (text, billboard) in (&mut ui_text, &mut billboard).join() {
            text.text = billboard.entire_text.chars().take(billboard.head).collect();
            billboard.head = if billboard.head == billboard.entire_text.len() - 1 {
                0
            } else {
                billboard.head + 1
            };
        }
    }
}
