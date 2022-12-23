use crate::plugin::billboard::BillboardData;
use crate::plugin::{despawn_with, Action, GameState, Goto, BTN_HEIGHT, GUTTER_V};
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub struct ChoicePlugin;

impl Plugin for ChoicePlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Choice, setup_choices)
            .add_exit_system(GameState::Choice, teardown_choices)
            .add_exit_system(GameState::Choice, despawn_with::<ChoiceList>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Choice)
                    .with_system(choice_cursor_system)
                    .with_system(handle_choice_input)
                    .into(),
            );
    }
}

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

fn setup_choices(mut commands: Commands, choices: Res<Choices>, ass: Res<AssetServer>) {
    let style = TextStyle {
        font: ass.load("CC Accidenz Commons-medium.ttf"),
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
                parent.spawn((txt, Choice));
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

/// Resource used to build a menu of choices.
#[derive(Resource)]
pub struct Choices(pub Vec<crate::talkie_core::Choice>);

#[derive(Component)]
struct ChoiceCursor;

#[derive(Component)]
struct ChoiceList {
    selected_choice: usize,
    choices: Vec<crate::talkie_core::Choice>,
}

#[derive(Component, Debug)]
struct Choice;
