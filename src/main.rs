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

/// The default number of glyphs to reveal per second.
///
/// This value is used as a fallback for when the `TALKIE_SPEED` env var is
const DEFAULT_GLYPHS_PER_SEC: f32 = 18.0;
const TALKIE_SPEED_FACTOR: f32 = 30.0;

/// Our Application State
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    Loading,
    Playback,
    Prompt,
    Choice,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_loopless_state(GameState::Loading)
        .add_fixed_timestep(
            Duration::from_millis(125),
            // give it a label
            "my_fixed_update",
        )
        // menu setup (state enter) systems
        .add_enter_system(GameState::Loading, setup_billboard)
        .add_enter_system(GameState::Choice, setup_choices)
        .add_exit_system(GameState::Choice, teardown_choices)
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

// XXX: most fields were lifted from `states::BillboardData`.
// The amethyst version held a handle to an asset for the dialogue data.
// This should probably be true here as well, but inlining the data to start.
#[derive(Component, Debug, Default)]
struct BillboardData {
    /// Tracks the current length of *displayed text*.
    head: usize,
    /// tracks which passage group we're iterating through.
    passage_group: usize,
    /// tracks which passage we're showing.
    passage: usize,
    /// Tracks the time since the last glyph was revealed.
    // XXX: We could default this to 0.0 and not bother with the Option, but
    //  I thought it might be interesting to be able to know when we're starting
    //  totally from scratch vs rolling over from a previous iteration.
    secs_since_last_reveal: Option<f32>,
    // FIXME: used to be a field on PlaybackState, but the states themselves are now represented as enums...
    fast_forward: bool,
    // FIXME: used to be a field on PlaybackState, but the states themselves are now represented as enums...
    glyphs_per_sec: f32,
}

#[derive(Component)]
struct SpeakerNameText;

#[derive(Component)]
struct DialogueText;

/// Resource used to build a menu of choices.
#[derive(Resource)]
struct Choices(Vec<core::Choice>);

/// Resource used to build a menu of choices.
#[derive(Resource)]
struct Dialogue(core::Dialogue);

#[derive(Component)]
struct ChoiceList {
    selected_choice: usize,
}

/// Resource used to signal a jump to a given passage group.
struct Goto(String);

#[derive(Component)]
struct GameCamera;

// FIXME: body should resemble PlaybackState::fixed_update
fn playback_system(
    mut commands: Commands,
    time: Res<Time>,
    dialogue: Res<Dialogue>,
    mut billboard: Query<&mut BillboardData>,
    mut params: ParamSet<(
        Query<(&mut Text, With<SpeakerNameText>)>,
        Query<(&mut Text, With<DialogueText>)>,
    )>,
) {
    let mut billboard = billboard.single_mut();

    // TODO: check input to see if fast-forward should be set

    let group = &dialogue.0.passage_groups[billboard.passage_group];

    // XXX: text/passages should not end up empty. If they are, it
    // there be a problem with the parser.
    let entire_text = group.passages[billboard.passage].as_str();

    if billboard.head < entire_text.len() {
        {
            let mut q = params.p0();
            let (mut txt, _) = q.single_mut();
            txt.sections[0].value = group
                .speaker
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("")
                .to_string();
        }

        let mut since = billboard.secs_since_last_reveal.unwrap_or_default();

        since += time.delta_seconds();

        let (reveal_how_many, remainder) = core::calc_glyphs_to_reveal(
            since,
            billboard.glyphs_per_sec
                * if billboard.fast_forward {
                    TALKIE_SPEED_FACTOR
                } else {
                    1.0
                },
        );

        billboard.secs_since_last_reveal = Some(remainder);
        billboard.head += reveal_how_many; // Only advance if we can update the display
        {
            let mut q = params.p1();
            let (mut txt, _) = q.single_mut();
            txt.sections[0].value = entire_text.chars().take(billboard.head).collect();
        }
    } else {
        let last_group = billboard.passage_group == dialogue.0.passage_groups.len() - 1;
        let last_passage = billboard.passage == group.passages.len() - 1;

        // Go back to the very start if we're at the end of the last
        // passage.
        // If we're at the end of any other passage, reset the head
        // but advance to the next passage.
        // Otherwise, reveal another glyph of the current passage.
        match (last_passage, last_group) {
            (true, true) => {
                billboard.head = 0;
                billboard.passage_group = 0;
                billboard.passage = 0;
            }
            (false, _) => {
                billboard.head = 0;
                billboard.passage += 1;
            }
            (true, false) => {
                billboard.head = 0;
                billboard.passage_group += 1;
                billboard.passage = 0;
            }
        }

        let has_choices = group
            .choices
            .as_ref()
            .map(|v| !v.is_empty())
            .unwrap_or(false);

        if last_passage && has_choices {
            commands.insert_resource(Choices(group.choices.clone().expect("choices")));
            commands.insert_resource(NextState(GameState::Choice));
            return;
        } else {
            // XXX: prompt state used to accept the name of the action to trigger on
            commands.insert_resource(NextState(GameState::Prompt));
        }
    }
}

fn prompt_system(mut _commands: Commands) {
    // TODO
}

fn choice_system(mut _commands: Commands, choices: Res<Choices>) {
    // TODO
}

fn setup_choices(mut commands: Commands, choices: Res<Choices>, ass: Res<AssetServer>) {
    let style = TextStyle {
        font: ass.load("Sansation-Regular.ttf"),
        font_size: 18.0,
        color: Color::BLACK,
    };

    let menu = commands
        .spawn(NodeBundle {
            background_color: BackgroundColor(Color::rgb(0.5, 0.55, 0.5)),
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                align_self: AlignSelf::Auto,
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for choice in &choices.0 {
                parent.spawn(TextBundle::from_section(&choice.label, style.clone()));
            }
        })
        .insert(ChoiceList { selected_choice: 0 })
        .id();

    commands.entity(menu);
}

// XXX: Might not be needed if we can cleanup in the `choice_system`
fn teardown_choices(mut commands: Commands) {
    commands.remove_resource::<Choices>();
}

/// We can just access the `CurrentState`, and even use change detection!
fn debug_current_state(state: Res<CurrentState<GameState>>) {
    if state.is_changed() {
        println!("Detected state change to {:?}!", state);
    }
}

/// Spawn the camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(GameCamera);
}

const BILLBOARD_HEIGHT: f32 = 300.0;

/// Construct the main conversation UI
fn setup_billboard(mut commands: Commands, ass: Res<AssetServer>) {
    // TODO: load sprites

    // In amethyst dialogue text and speaker name text were two separate UI
    // entities, handed off when constructing the playback state.
    // Once all the assets were loaded, the playback state is initialized and
    // the state machine transitioned to it.

    let style = TextStyle {
        font: ass.load("Sansation-Regular.ttf"),
        font_size: 24.0,
        color: Color::WHITE,
    };

    let billboard = commands
        .spawn(NodeBundle {
            background_color: BackgroundColor(Color::rgb(0.4, 0.4, 0.6)),
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                align_self: AlignSelf::Auto,
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle::from_section("speaker name", style.clone()))
                .insert(SpeakerNameText);

            parent
                .spawn(TextBundle::from_section("dialogue", style.clone()))
                .insert(DialogueText);
        })
        .insert(BillboardData {
            head: 0,
            passage_group: 0,
            passage: 0,
            secs_since_last_reveal: None,
            fast_forward: false,
            glyphs_per_sec: std::env::var("TALKIE_SPEED")
                .map(|s| s.parse().expect("invalid speed."))
                .unwrap_or(DEFAULT_GLYPHS_PER_SEC),
        })
        .id();

    commands.entity(billboard);

    // FIXME: when using asset loader, might need a handle instead of local data
    let dialogue_file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/dialogue/choices.toml");
    commands.insert_resource(Dialogue(
        core::Dialogue::from_path(dialogue_file_path).expect("load dialogue"),
    ));
    commands.insert_resource(NextState(GameState::Playback));
}

mod core;
