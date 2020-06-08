use crate::assets::dialogue::{DialogueFormat, DialogueHandle};
use amethyst::{
    assets::Loader,
    ecs::{
        prelude::{Component, HashMapStorage},
        World,
    },
    prelude::*,
    ui::{Anchor, LineMode, TtfFormat, UiText, UiTransform},
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
    let font = world.read_resource::<Loader>().load(
        "font/CC Accidenz Commons-medium.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );

    let dialogue = world.read_resource::<Loader>().load(
        "dialogue/lipsum.dialogue",
        DialogueFormat,
        (),
        &world.read_resource(),
    );

    let xform = UiTransform::new(
        "text".to_string(),
        Anchor::BottomLeft,
        Anchor::BottomLeft,
        5.,
        5.,
        1.,
        // based on a 500x500 window
        490.,
        250.,
    );

    let mut ui_text = UiText::new(font, String::new(), [1., 1., 1., 1.], 24.);
    ui_text.line_mode = LineMode::Wrap;
    ui_text.align = Anchor::TopLeft;

    let billboard = world
        .create_entity()
        .with(xform)
        .with(ui_text)
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
