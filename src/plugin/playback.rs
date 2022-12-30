use crate::plugin::billboard::{Bookmark, DialogueText, PlayHead, SpeakerNameTab, SpeakerNameText};
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
                .with_system(input_handler)
                .with_system(reveal_timer_reset)
                .with_system(playback_system)
                .into(),
        );
    }
}

fn input_handler(
    action_state: Query<&ActionState<Action>, With<PlayHead>>,
    mut playhead: Query<&mut PlayHead>,
) {
    let action_state = action_state.single();
    let mut playhead = playhead.single_mut();
    playhead.fast_forward = action_state.pressed(Action::Confirm);
}

fn reveal_timer_reset(mut query: Query<&mut PlayHead, Changed<Bookmark>>) {
    if let Ok(mut playhead) = query.get_single_mut() {
        println!("Resetting playhead last reveal time");
        playhead.secs_since_last_reveal = None;
    }
}

#[allow(clippy::type_complexity)]
fn playback_system(
    mut commands: Commands,
    time: Res<Time>,
    dialogue: Res<Dialogue>,
    mut billboard: Query<(&mut PlayHead, &mut Bookmark)>,
    mut display: ParamSet<(
        Query<(&mut Visibility, With<SpeakerNameTab>)>,
        Query<(&mut Text, With<SpeakerNameText>)>,
        Query<(&mut Text, With<DialogueText>)>,
    )>,
) {
    let (mut playhead, mut bookmark) = billboard.single_mut();
    let group = &dialogue.0.passage_groups[bookmark.passage_group];
    let entire_text = group.passages[bookmark.passage].as_str();

    if playhead.head < entire_text.len() {
        {
            // TODO: refactor so we only do this when the passage group is changing
            //  Speaker names are by passage group so doing this every tick is needless.
            let speaker_name = group.speaker.as_deref().unwrap_or("");
            {
                let mut t = display.p1();
                let (mut txt, _) = t.single_mut();
                txt.sections[0].value = speaker_name.to_string();
            }
            let mut v = display.p0();
            let (mut vis, _) = v.single_mut();
            vis.is_visible = !speaker_name.trim().is_empty();
        }

        let mut since = playhead.secs_since_last_reveal.unwrap_or_default();
        since += time.delta_seconds();

        let (reveal_how_many, remainder) = crate::talkie_core::calc_glyphs_to_reveal(
            since,
            playhead.glyphs_per_sec
                * if playhead.fast_forward {
                    TALKIE_SPEED_FACTOR
                } else {
                    1.0
                },
        );

        playhead.secs_since_last_reveal = Some(remainder);
        playhead.head += reveal_how_many; // Only advance if we can update the display
        {
            let mut q = display.p2();
            let (mut txt, _) = q.single_mut();
            txt.sections[0].value = entire_text.chars().take(playhead.head).collect();
        }
    } else {
        let last_group = bookmark.passage_group == dialogue.0.passage_groups.len() - 1;
        let last_passage = bookmark.passage == group.passages.len() - 1;

        // Go back to the very start if we're at the end of the last
        // passage.
        // If we're at the end of any other passage, reset the head
        // but advance to the next passage.
        // Otherwise, reveal another glyph of the current passage.
        match (last_passage, last_group) {
            (true, true) => {
                playhead.head = 0;
                bookmark.passage_group = 0;
                bookmark.passage = 0;
            }
            (false, _) => {
                playhead.head = 0;
                bookmark.passage += 1;
            }
            (true, false) => {
                playhead.head = 0;
                bookmark.passage_group += 1;
                bookmark.passage = 0;
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
            commands.insert_resource(NextState(GameState::Prompt));
        }
    }
}
