use crate::plugin::billboard::{Billboard, Bookmark};
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
    mut bookmark: Query<&mut Bookmark>,
    billboard: Query<&Billboard>,
    dialogue: Res<Assets<Dialogue>>,
) {
    if let Some(passage_group_id) = goto.0.take() {
        let mut bookmark = bookmark.single_mut();
        println!("Got goto={passage_group_id}");

        let billboard = billboard.single();
        let dialogue = dialogue.get(&billboard.dialogue).expect("dialogue");

        bookmark.passage_group = dialogue
            .0
            .passage_groups
            .iter()
            .position(|group| group.id.as_ref() == Some(&passage_group_id))
            .unwrap();
    } else {
        println!("Got goto=Next");
    }
    commands.insert_resource(NextState(GameState::Playback));
}

/// Resource used to signal a jump to a given passage group.
#[derive(Resource)]
pub struct Goto(pub Option<String>);
