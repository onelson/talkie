use bevy::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;

mod billboard;
mod choice;
mod playback;
mod prompt;

pub struct TalkiePlugin;

impl Plugin for TalkiePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<Action>::default())
            .add_loopless_state(GameState::Loading)
            .add_plugin(billboard::BillboardPlugin)
            .add_plugin(choice::ChoicePlugin)
            .add_plugin(prompt::PromptPlugin)
            .add_plugin(playback::PlaybackPlugin)
            .add_system(debug_current_state);
    }
}

/// The default number of glyphs to reveal per second.
///
/// This value is used as a fallback for when the `TALKIE_SPEED` env var is
const DEFAULT_GLYPHS_PER_SEC: f32 = 14.0;
const TALKIE_SPEED_FACTOR: f32 = 10.0;

/// Our Application State
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    Loading,
    Playback,
    Prompt,
    Choice,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Action {
    Confirm,
    Up,
    Down,
}

/// Despawn all entities with a given component type
fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

/// Resource used to build a menu of choices.
#[derive(Resource)]
struct Dialogue(crate::talkie_core::Dialogue);

/// Resource used to signal a jump to a given passage group.
#[derive(Resource)]
pub struct Goto(Option<String>);

const GUTTER_V: f32 = 4.;
const BTN_HEIGHT: f32 = 28.;

/// We can just access the `CurrentState`, and even use change detection!
fn debug_current_state(state: Res<CurrentState<GameState>>) {
    if state.is_changed() {
        println!("Detected state change to {state:?}!");
    }
}
