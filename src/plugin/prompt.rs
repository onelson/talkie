use crate::plugin::billboard::BillboardData;
use crate::plugin::{despawn_with, Action, GameState, BTN_HEIGHT};
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub struct PromptPlugin;

impl Plugin for PromptPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Prompt, setup_prompt)
            .add_exit_system(GameState::Prompt, despawn_with::<PromptCursor>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Prompt)
                    .with_system(prompt_system)
                    .into(),
            );
    }
}

fn prompt_system(mut commands: Commands, query: Query<&ActionState<Action>, With<BillboardData>>) {
    let action_state = query.single();
    if action_state.just_pressed(Action::Confirm) {
        commands.insert_resource(NextState(GameState::Playback));
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

#[derive(Component)]
struct PromptCursor;
