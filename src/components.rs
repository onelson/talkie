use crate::assets::dialogue::DialogueHandle;
use amethyst::ecs::prelude::{Component, HashMapStorage};

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
    /// tracks the time since the last glyph was revealed.
    // XXX: We could default this to 0.0 and not bother with the Option, but
    //  I thought it might be interesting to be able to know when we're starting
    //  totally from scratch vs rolling over from a previous iteration.
    pub secs_since_last_reveal: Option<f32>,
}

impl Component for BillboardData {
    type Storage = HashMapStorage<Self>;
}
