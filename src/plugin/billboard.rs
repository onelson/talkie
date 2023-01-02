//! Ah. Well. At this point the "billboard" jargon is losing traction.
//! Still, the general idea was a billboard is the top-level or entrypoint for
//! the whole dialogue presentation.

use crate::plugin::goto::Goto;
use crate::plugin::{Action, Dialogue, GameState, DEFAULT_GLYPHS_PER_SEC};
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct BillboardPlugin;

impl Plugin for BillboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Loading, setup_billboard)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Loading)
                    .with_system(wait_for_assets)
                    .into(),
            );
    }
}

#[derive(Component, Debug, Default)]
pub struct Bookmark {
    /// tracks which passage group we're iterating through.
    pub passage_group: usize,
    /// tracks which passage we're showing.
    pub passage: usize,
}

#[derive(Component, Debug, Default)]
pub struct PlayHead {
    /// Tracks the current length of *displayed text*.
    pub head: usize,
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
pub struct SpeakerNameTab;

#[derive(Component)]
pub struct SpeakerNameText;

#[derive(Component)]
pub struct DialogueText;

#[derive(Component)]
pub struct Billboard {
    pub dialogue: Handle<Dialogue>,
}

#[derive(Component)]
pub struct Root;

const BILLBOARD_HEIGHT: Val = Val::Px(300.0);

fn wait_for_assets(mut commands: Commands, ass: Res<Assets<Dialogue>>, query: Query<&Billboard>) {
    if let Ok(b) = query.get_single() {
        if ass.get(&b.dialogue).is_some() {
            commands.insert_resource(NextState(GameState::Playback));
            commands.insert_resource(Goto(None));
        }
    }
}

/// Construct the main conversation UI
fn setup_billboard(mut commands: Commands, ass: Res<AssetServer>) {
    let dialogue = ass.load("dialogue/mgs3-body-snatchers.toml");

    // TODO: load sprites

    // In amethyst dialogue text and speaker name text were two separate UI
    // entities, handed off when constructing the playback state.
    // Once all the assets were loaded, the playback state is initialized and
    // the state machine transitioned to it.

    let style = TextStyle {
        font: ass.load("Sansation-Regular.ttf"),
        font_size: 20.0,
        color: Color::WHITE,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::ColumnReverse,
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
            let billboard_color = Color::rgb(0.4, 0.4, 0.6);
            parent
                .spawn(NodeBundle {
                    background_color: BackgroundColor(billboard_color),
                    style: Style {
                        position: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Auto, Val::Px(0.0)),
                        padding: UiRect::all(Val::Px(20.0)),
                        margin: UiRect::all(Val::Px(20.0)),
                        size: Size::new(Val::Auto, BILLBOARD_HEIGHT),
                        ..default()
                    },
                    ..default()
                })
                .insert(Root)
                .with_children(|parent| {
                    let name_tab = NodeBundle {
                        // N.b. systems should manage the visibility per passage
                        visibility: Visibility::INVISIBLE,
                        background_color: BackgroundColor(billboard_color),
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect::bottom(BILLBOARD_HEIGHT),
                            padding: UiRect::all(Val::Px(14.0)),
                            border: UiRect::all(Val::Px(4.0)),
                            ..default()
                        },
                        ..default()
                    };

                    parent
                        .spawn(name_tab)
                        .with_children(|parent| {
                            parent
                                .spawn(TextBundle::from_section("speaker name", style.clone()))
                                .insert(SpeakerNameText);
                        })
                        .insert(SpeakerNameTab);

                    let mut text = TextBundle::from_section("dialogue", style.clone());
                    text.style.position_type = PositionType::Absolute;
                    parent.spawn(text).insert(DialogueText);
                });
        })
        .insert((
            PlayHead {
                head: 0,
                secs_since_last_reveal: None,
                fast_forward: false,
                glyphs_per_sec: std::env::var("TALKIE_SPEED")
                    .map(|s| s.parse().expect("invalid speed."))
                    .unwrap_or(DEFAULT_GLYPHS_PER_SEC),
            },
            Bookmark::default(),
            Billboard { dialogue },
        ));
}
