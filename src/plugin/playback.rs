use crate::plugin::billboard::{BillboardData, DialogueText, SpeakerNameText};
use crate::plugin::choice::Choices;
use crate::plugin::{Action, Dialogue, GameState, TALKIE_SPEED_FACTOR};
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub struct PlaybackPlugin;

impl Plugin for PlaybackPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playback)
                .with_system(playback_system)
                .into(),
        );
    }
}

#[allow(clippy::type_complexity)]
fn playback_system(
    mut commands: Commands,
    time: Res<Time>,
    dialogue: Res<Dialogue>,
    action_state: Query<&ActionState<Action>, With<BillboardData>>,
    mut billboard: Query<&mut BillboardData>,
    mut params: ParamSet<(
        Query<(&mut Text, With<SpeakerNameText>)>,
        Query<(&mut Text, With<DialogueText>)>,
    )>,
) {
    let mut billboard = billboard.single_mut();

    billboard.fast_forward = action_state.single().pressed(Action::Confirm);

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

        let (reveal_how_many, remainder) = crate::talkie_core::calc_glyphs_to_reveal(
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
