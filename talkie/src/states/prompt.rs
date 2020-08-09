use crate::components::ActionTracker;
use amethyst::core::ecs::{Entity, WorldExt};
use amethyst::core::Hidden;
use amethyst::input::{InputHandler, StringBindings};
use amethyst::ui::UiFinder;
use amethyst::{GameData, SimpleState, SimpleTrans, StateData, Trans};

// FIXME: add a bool field for "hidden" which will drive the entity modifications over time.
//  By driving the hide/show with a separate field on the state, we can do things
//  like blink it or whatever.
pub struct PromptState {
    icon: Option<Entity>,
    tracker: ActionTracker,
}

impl PromptState {
    pub fn new(action: &str) -> PromptState {
        PromptState {
            icon: None,
            tracker: ActionTracker::new(action),
        }
    }
}

impl SimpleState for PromptState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        if self.icon.is_none() {
            world.exec(|ui_finder: UiFinder| {
                self.icon = ui_finder.find("next_page");
            });
        }

        let input = world.read_resource::<InputHandler<StringBindings>>();
        // By updating the tracker on start, we maintain continuity with the
        // input state of the previous state.
        // This is important because we want to *transition out of this state*
        // only when the button has been released and re-pressed.
        // Without this initial update, the player could just hold the button
        // down to advance through all passages.
        self.tracker.update(&input);
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        if let Some(icon) = self.icon {
            let mut storage = data.world.write_storage::<Hidden>();
            let _ = storage.insert(icon, Hidden);
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        match self.icon {
            None => return Trans::None,
            Some(icon) => {
                let mut storage = data.world.write_storage::<Hidden>();
                let _ = storage.remove(icon);
            }
        }

        let input = data.world.read_resource::<InputHandler<StringBindings>>();

        self.tracker.update(&input);

        if self.tracker.press_begin() {
            Trans::Pop
        } else {
            Trans::None
        }
    }
}
