use crate::plugin::billboard::BillboardData;
use crate::plugin::{Dialogue, GameState};
use bevy::prelude::*;
use iyes_loopless::prelude::*;

pub struct GotoPlugin;

impl Plugin for GotoPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Goto)
                .with_system(goto_system)
                .into(),
        );
    }
}

fn goto_system(
    mut commands: Commands,
    mut goto: ResMut<Goto>,
    mut billboard: Query<&mut BillboardData>,
    dialogue: Res<Dialogue>,
) {
    let mut billboard = billboard.single_mut();
    if let Some(passage_group_id) = goto.0.take() {
        println!("Got goto={passage_group_id}");
        billboard.passage_group = dialogue
            .0
            .passage_groups
            .iter()
            .position(|group| group.id.as_ref() == Some(&passage_group_id))
            .unwrap();
    } else {
        println!("Got goto=Next");
    }
    billboard.secs_since_last_reveal = None;
    commands.insert_resource(NextState(GameState::Playback));
}

/// Resource used to signal a jump to a given passage group.
#[derive(Resource)]
pub struct Goto(Option<String>);
