use amethyst::{
    ecs::prelude::{Component, HashMapStorage},
    input::{InputHandler, StringBindings},
};
use log::debug;

#[derive(Debug, Clone)]
pub struct ActionTracker {
    action: String,
    /// True only when the action is pressed after not being pressed previously.
    press_begin: bool,
    /// True from the very first press up until the press ends.
    pressed: bool,
    /// Marks the point where the action was just released.
    press_end: bool,
}

impl ActionTracker {
    pub fn new<S: Into<String>>(action: S) -> Self {
        Self {
            action: action.into(),
            press_begin: false,
            pressed: false,
            press_end: false,
        }
    }

    pub fn press_begin(&self) -> bool {
        self.press_begin
    }

    pub fn pressed(&self) -> bool {
        self.pressed
    }

    pub fn press_end(&self) -> bool {
        self.press_end
    }

    pub fn update(&mut self, input: &()) {
        let is_down = input.action_is_down(&self.action).unwrap_or(false);
        match (is_down, self.pressed) {
            (true, false) => {
                self.press_begin = true;
                self.pressed = true;
                self.press_end = false;
                debug!("{}=down", &self.action);
            }
            (true, true) => {
                self.press_begin = false;
                self.pressed = true;
                self.press_end = false;
                debug!("{}=pressed", &self.action);
            }
            (false, true) => {
                self.press_begin = false;
                self.pressed = false;
                self.press_end = true;
                debug!("{}=up", &self.action);
            }
            (false, false) => {
                self.press_begin = false;
                self.pressed = false;
                self.press_end = false;
            }
        }
    }
}
