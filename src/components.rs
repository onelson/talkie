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
