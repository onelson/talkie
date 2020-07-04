use crate::components;
use amethyst::prelude::{GameData, SimpleState, StateData};

pub struct MyState;

impl SimpleState for MyState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        components::init_billboard(data.world);
    }
}
