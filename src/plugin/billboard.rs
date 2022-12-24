use crate::plugin::goto::Goto;
use crate::plugin::{Action, Dialogue, GameState, DEFAULT_GLYPHS_PER_SEC};
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct BillboardPlugin;

impl Plugin for BillboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Loading, setup_billboard);
    }
}

#[derive(Component, Debug, Default)]
pub struct BillboardData {
    /// Tracks the current length of *displayed text*.
    pub head: usize,
    /// tracks which passage group we're iterating through.
    pub passage_group: usize,
    /// tracks which passage we're showing.
    pub passage: usize,
    /// Tracks the time since the last glyph was revealed.
    // XXX: We could default this to 0.0 and not bother with the Option, but
    //  I thought it might be interesting to be able to know when we're starting
    //  totally from scratch vs rolling over from a previous iteration.
    // FIXME: resetting this currently happens in a couple places.
    //   If we isolate the passage_group and passage fields on their own component we could watch
    //   it for changes in the playback system and reset this field at that point.
    pub secs_since_last_reveal: Option<f32>,
    pub fast_forward: bool,
    pub glyphs_per_sec: f32,
}

#[derive(Component)]
pub struct SpeakerNameText;

#[derive(Component)]
pub struct DialogueText;

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
        crate::talkie_core::Dialogue::from_path(dialogue_file_path).expect("load dialogue"),
    ));
    commands.insert_resource(NextState(GameState::Playback));
    commands.insert_resource(Goto(None));
}
