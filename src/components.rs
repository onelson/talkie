use crate::assets::dialogue::{DialogueFormat, DialogueHandle};
use amethyst::{
    assets::Loader,
    ecs::{
        prelude::{Component, HashMapStorage},
        World,
    },
    prelude::*,
    ui::UiCreator,
};

#[derive(Debug, Clone)]
pub struct ActionTracker {
    pub action: &'static str,
    /// True only when the action is pressed after not being pressed previously.
    pub press_begin: bool,
    /// True from the very first press up until the press ends.
    pub pressed: bool,
    /// Marks the point where the action was just released.
    pub press_end: bool,
}

impl ActionTracker {
    pub fn new(action: &'static str) -> Self {
        Self {
            action,
            press_begin: false,
            pressed: false,
            press_end: false,
        }
    }
}

impl Component for ActionTracker {
    type Storage = HashMapStorage<Self>;
}

/// The "billboard" is the lower half of the window where dialogue text will
/// appear.
pub struct BillboardData {
    pub dialogue: DialogueHandle,
    /// tracks the current length of *displayed text*.
    pub head: usize,
    /// tracks which passage group we're iterating through.
    pub passage_group: usize,
    /// tracks which passage we're showing.
    pub passage: usize,
    pub paused: bool,
}

impl Component for BillboardData {
    type Storage = HashMapStorage<Self>;
}

pub fn init_billboard(world: &mut World) {
    let dialogue = world.read_resource::<Loader>().load(
        "dialogue/lipsum.dialogue",
        DialogueFormat,
        (),
        &world.read_resource(),
    );

    world.exec(|mut creator: UiCreator<'_>| {
        creator.create("billboard.ron", ());
    });

    let billboard = world
        .create_entity()
        .with(BillboardData {
            dialogue,
            head: 0,
            passage_group: 0,
            passage: 0,
            paused: false,
        })
        .with(ActionTracker::new("confirm"))
        .build();

    world.insert(billboard);
}
