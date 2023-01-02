use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::Deserialize;

mod billboard;
mod choice;
mod goto;
mod playback;
mod prompt;

pub struct TalkiePlugin;

impl Plugin for TalkiePlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Dialogue>()
            .init_asset_loader::<DialogueLoader>()
            .add_plugin(InputManagerPlugin::<Action>::default())
            .add_loopless_state(GameState::Loading)
            .add_plugin(billboard::BillboardPlugin)
            .add_plugin(choice::ChoicePlugin)
            .add_plugin(goto::GotoPlugin)
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
    Choice,
    Goto,
    Playback,
    Prompt,
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
#[derive(Debug, Deserialize, TypeUuid)]
#[uuid = "75348891-801a-447f-9663-0f08e0247859"]
pub struct Dialogue(crate::talkie_core::Dialogue);

/// We can just access the `CurrentState`, and even use change detection!
fn debug_current_state(state: Res<CurrentState<GameState>>) {
    if state.is_changed() {
        println!("Detected state change to {state:?}!");
    }
}

#[derive(Default)]
pub struct DialogueLoader;

impl AssetLoader for DialogueLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let dialogue = crate::talkie_core::Dialogue::from_slice(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(Dialogue(dialogue)));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}
