use bevy::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;
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

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Confirm,
    Up,
    Down,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
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
        .add_exit_system(GameState::Choice, despawn_with::<ChoiceList>)
        .add_enter_system(GameState::Prompt, setup_prompt)
        .add_exit_system(GameState::Prompt, despawn_with::<PromptCursor>)
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
                .with_system(choice_cursor_system)
                .with_system(handle_choice_input)
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

/// Despawn all entities with a given component type
fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

#[derive(Component)]
struct SpeakerNameText;

#[derive(Component)]
struct DialogueText;

/// Resource used to build a menu of choices.
#[derive(Resource)]
struct Choices(Vec<talkie_core::Choice>);

/// Resource used to build a menu of choices.
#[derive(Resource)]
struct Dialogue(talkie_core::Dialogue);

#[derive(Component)]
struct ChoiceCursor;

#[derive(Component)]
struct PromptCursor;

#[derive(Component)]
struct ChoiceList {
    selected_choice: usize,
    choices: Vec<talkie_core::Choice>,
}

#[derive(Component, Debug)]
struct Choice {
    idx: usize,
}

/// Resource used to signal a jump to a given passage group.
#[derive(Resource)]
struct Goto(Option<String>);

#[derive(Component)]
struct GameCamera;

// FIXME: body should resemble PlaybackState::fixed_update
fn playback_system(
    mut commands: Commands,
    time: Res<Time>,
    dialogue: Res<Dialogue>,
    mut goto: ResMut<Goto>,
    mut billboard: Query<&mut BillboardData>,
    mut params: ParamSet<(
        Query<(&mut Text, With<SpeakerNameText>)>,
        Query<(&mut Text, With<DialogueText>)>,
    )>,
) {
    let mut billboard = billboard.single_mut();

    // XXX: formerly we'd check for a GOTO here and reset the BillboardData when present.
    // Might actually be better to do this in a separate state and/or system.
    // Could be written to reset the playhead, then transition back here.
    if goto.is_changed() {
        if let Some(passage_group_id) = goto.0.take() {
            println!("Got goto={passage_group_id}");
            billboard.passage_group = dialogue
                .0
                .passage_groups
                .iter()
                .position(|group| group.id.as_ref() == Some(&passage_group_id))
                .unwrap();
        }
    }

    // TODO: check input to see if fast-forward should be set

    let group = &dialogue.0.passage_groups[billboard.passage_group];

    // XXX: text/passages should not end up empty. If they are, it
    // there be a problem with the parser.
    let entire_text = group.passages[billboard.passage].as_str();

    if billboard.head < entire_text.len() {
        {
            let mut q = params.p0();
            let (mut txt, _) = q.single_mut();
            txt.sections[0].value = group.speaker.as_deref().unwrap_or("").to_string();
        }

        let mut since = billboard.secs_since_last_reveal.unwrap_or_default();

        since += time.delta_seconds();

        let (reveal_how_many, remainder) = talkie_core::calc_glyphs_to_reveal(
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
        } else {
            // XXX: prompt state used to accept the name of the action to trigger on
            commands.insert_resource(NextState(GameState::Prompt));
        }
    }
}

fn prompt_system(mut commands: Commands, query: Query<&ActionState<Action>, With<BillboardData>>) {
    let action_state = query.single();
    if action_state.pressed(Action::Confirm) {
        commands.insert_resource(NextState(GameState::Playback));
    }
}

const GUTTER_V: f32 = 4.;
const BTN_HEIGHT: f32 = 28.;
const BTN_WIDTH: f32 = 100.;

fn choice_cursor_system(
    mut _commands: Commands,
    choice_list: Query<&ChoiceList>,
    mut choice_cursor: Query<&mut Style, With<ChoiceCursor>>,
) {
    let choice_list = choice_list.single();
    let v_offset = choice_list.choices.len() - choice_list.selected_choice;
    let mut cursor_style = choice_cursor.single_mut();
    cursor_style.position = UiRect::bottom(Val::Px(
        (v_offset as f32 * (BTN_HEIGHT + GUTTER_V)) - GUTTER_V,
    ));
}

fn handle_choice_input(
    mut commands: Commands,
    mut choice_list: Query<&mut ChoiceList>,
    mut goto: ResMut<Goto>,
    query: Query<&ActionState<Action>, With<BillboardData>>,
) {
    let action_state = query.single();

    if action_state.just_pressed(Action::Confirm) {
        let choice_list = choice_list.single();
        goto.0 = choice_list.choices[choice_list.selected_choice]
            .goto
            .clone();
        commands.insert_resource(NextState(GameState::Playback));
        return;
    }

    let mut choice_list = choice_list.single_mut();
    if action_state.just_pressed(Action::Up) && choice_list.selected_choice > 0 {
        choice_list.selected_choice -= 1;
    }
    if action_state.just_pressed(Action::Down)
        && choice_list.selected_choice < choice_list.choices.len() - 1
    {
        choice_list.selected_choice += 1;
    }
}

fn setup_prompt(mut commands: Commands, _ass: Res<AssetServer>) {
    let cursor = commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(BTN_HEIGHT), Val::Px(BTN_HEIGHT)),
                    position_type: PositionType::Absolute,
                    position: UiRect::new(Val::Auto, Val::Px(0.), Val::Auto, Val::Px(0.)),
                    ..default()
                },
                background_color: Color::rgb(0.9, 0.3, 0.3).into(),
                ..default()
            },
            PromptCursor,
        ))
        .id();
    commands.entity(cursor);
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
                align_content: AlignContent::FlexEnd,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(BTN_HEIGHT), Val::Px(BTN_HEIGHT)),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    background_color: Color::rgb(0.9, 0.3, 0.3).into(),
                    ..default()
                },
                ChoiceCursor,
            ));

            let choice_count = choices.0.len();
            for (idx, choice) in choices.0.iter().enumerate() {
                // We need to calculate the offset from the end of the list in order
                // to get these positioned correctly when pinned to the bottom.
                let v_offset = choice_count - idx;
                // FIXME: might need to wrap each in a node to give some padding
                let mut txt = TextBundle::from_section(&choice.label, style.clone());
                txt.style.position_type = PositionType::Absolute;
                txt.style.position =
                    UiRect::bottom(Val::Px(v_offset as f32 * (BTN_HEIGHT + GUTTER_V)));
                parent.spawn((txt, Choice { idx }));
            }
        })
        .insert(ChoiceList {
            selected_choice: 0,
            choices: choices.0.clone(),
        })
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
        println!("Detected state change to {state:?}!");
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
        .spawn((
            NodeBundle {
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
            },
            InputManagerBundle::<Action> {
                // Stores "which actions are currently pressed"
                action_state: ActionState::default(),
                // Describes how to convert from player inputs into those actions
                input_map: InputMap::new([
                    (KeyCode::Space, Action::Confirm),
                    (KeyCode::Return, Action::Confirm),
                    (KeyCode::Up, Action::Up),
                    (KeyCode::W, Action::Up),
                    (KeyCode::Down, Action::Down),
                    (KeyCode::S, Action::Down),
                ]),
            },
        ))
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
        talkie_core::Dialogue::from_path(dialogue_file_path).expect("load dialogue"),
    ));
    commands.insert_resource(NextState(GameState::Playback));
    commands.insert_resource(Goto(None));
}

mod talkie_core;
