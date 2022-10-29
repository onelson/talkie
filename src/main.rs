//! Complex example showcasing all the features together.
//!
//! Shows how our states, fixed timestep, and custom run conditions, can all be used together!
//!
//! Also shows how run conditions could be helpful for Bevy UI button handling!
//!
//! This example has a main menu with two buttons: exit the app and enter the game.
//!
//! How to "play the game": hold spacebar to spawn colorful squares, release spacebar to make them spin! <3

use bevy::prelude::*;
use iyes_loopless::prelude::*;
use std::time::Duration;

/// Our Application State
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    Loading,
    Playback,
    Prompt,
    Choice,
}

fn main() {
    // stage for anything we want to do on a fixed timestep
    let fixedupdate = SystemStage::parallel();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_loopless_state(GameState::Loading)
        .add_stage_before(
            CoreStage::Update,
            "FixedUpdate",
            FixedTimestepStage::from_stage(Duration::from_millis(125), fixedupdate),
        )
        // menu setup (state enter) systems
        .add_enter_system(GameState::Loading, setup_billboard)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playback)
                .with_system(playback_system)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Prompt)
                .with_system(prompt_system)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Choice)
                .with_system(choice_system)
                .into(),
        )
        // our other various systems:
        .add_system(debug_current_state)
        // setup our camera globally (for UI) at startup and keep it alive at all times
        .add_startup_system(setup_camera)
        .run();
}

#[derive(Component)]
struct Billboard;

#[derive(Component)]
struct SpeakerName;

#[derive(Component)]
struct Dialogue;

#[derive(Component)]
struct GameCamera;

fn playback_system(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::Prompt));
}

fn prompt_system(mut _commands: Commands) {
    // todo!("prompt");
}

fn choice_system(mut _commands: Commands) {
    // todo!("choice");
}

/// We can just access the `CurrentState`, and even use change detection!
fn debug_current_state(state: Res<CurrentState<GameState>>) {
    if state.is_changed() {
        println!("Detected state change to {:?}!", state);
    }
}

/// Spawn the camera
fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(GameCamera);
}

/// Construct the main conversation UI
fn setup_billboard(mut commands: Commands, ass: Res<AssetServer>) {
    // TODO: load sprites
    // TODO: store dialogue - separate component or entity?
    // TODO: store speaker name - separate component or entity?

    // In amethyst dialog text and speaker name text were two separate UI
    // entities, handed off when constructing the playback state.
    // Once all the assets were loaded, the playback state is initialized and
    // the state machine transitioned to it.

    let style = TextStyle {
        font: ass.load("Sansation-Regular.ttf"),
        font_size: 24.0,
        color: Color::WHITE,
    };
    let billboard = commands
        .spawn_bundle(NodeBundle {
            color: UiColor(Color::rgb(0.5, 0.5, 0.5)),
            style: Style {
                size: Size::new(Val::Auto, Val::Auto),
                margin: UiRect::all(Val::Auto),
                align_self: AlignSelf::Center,
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle::from_section("speaker name", style.clone()))
                .insert(SpeakerName);
            parent
                .spawn_bundle(TextBundle::from_section("dialogue", style.clone()))
                .insert(Dialogue);
        })
        .insert(Billboard)
        .id();

    commands.entity(billboard);
    commands.insert_resource(NextState(GameState::Playback));
}
