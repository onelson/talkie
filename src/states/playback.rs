use crate::assets::dialogue::{Dialogue, DialogueHandle};
use crate::components::ActionTracker;
use crate::states::choice::{ChoiceState, Goto};
use crate::states::prompt::PromptState;
use crate::states::BillboardData;
use crate::utils::calc_glyphs_to_reveal;
use amethyst::assets::AssetStorage;
use amethyst::core::ecs::{Builder, Entity, WorldExt};
use amethyst::core::Time;
use amethyst::input::{InputHandler, StringBindings};
use amethyst::renderer::Transparent;
use amethyst::ui::UiText;
use amethyst::{GameData, SimpleState, SimpleTrans, StateData, Trans};

/// The default number of glyphs to reveal per second.
///
/// This value is used as a fallback for when the `TALKIE_SPEED` env var is
/// unset while constructing a new `PlaybackState`.
const DEFAULT_GLYPHS_PER_SEC: f32 = 18.0;
const TALKIE_SPEED_FACTOR: f32 = 30.0;

/// Render the dialogue text over time.
pub struct PlaybackState {
    tracker: ActionTracker,
    dialogue_handle: DialogueHandle,
    /// The number of glyphs that should be revealed per second.
    glyphs_per_sec: f32,
    speaker_name_txt: Entity,
    dialogue_txt: Entity,
    /// When true, the text reveal speed is scaled up.
    fastforward: bool,
}

impl PlaybackState {
    pub fn new(
        dialogue_handle: DialogueHandle,
        speaker_name_txt: Entity,
        dialogue_txt: Entity,
    ) -> PlaybackState {
        PlaybackState {
            dialogue_handle,
            glyphs_per_sec: std::env::var("TALKIE_SPEED")
                .map(|s| s.parse().expect("invalid speed."))
                .unwrap_or(DEFAULT_GLYPHS_PER_SEC),
            speaker_name_txt,
            dialogue_txt,
            tracker: ActionTracker::new("confirm"),
            fastforward: false,
        }
    }
}

impl SimpleState for PlaybackState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.insert(Goto::default());
        world.insert(BillboardData {
            dialogue_id: self.dialogue_handle.id(),
            head: 0,
            passage_group: 0,
            passage: 0,
            secs_since_last_reveal: None,
        });

        let billboard = world.create_entity().with(Transparent).build();
        world.insert(billboard);
    }

    fn on_resume(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.fastforward = false; // reset

        let mut goto = data.world.try_fetch_mut::<Goto>().unwrap();
        if let Some(passage_group_id) = goto.passage_group_id.take() {
            log::debug!("Got goto={}", passage_group_id);
            let billboard = &mut data.world.write_resource::<BillboardData>();
            let dialogue_storage = data.world.read_resource::<AssetStorage<Dialogue>>();
            let dialogue = dialogue_storage
                .get_by_id(self.dialogue_handle.id())
                .unwrap();
            billboard.passage_group = dialogue
                .passage_groups
                .iter()
                .position(|group| group.id.as_ref() == Some(&passage_group_id))
                .unwrap();
        }
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let input = data.world.read_resource::<InputHandler<StringBindings>>();
        self.tracker.update(&input);

        if self.tracker.press_begin() {
            self.fastforward = true;
        }
        if self.tracker.press_end() {
            self.fastforward = false;
        }

        let billboard = &mut data.world.write_resource::<BillboardData>();
        let dialogue_storage = data.world.read_resource::<AssetStorage<Dialogue>>();
        let dialogue = dialogue_storage
            .get_by_id(self.dialogue_handle.id())
            .unwrap();
        let group = &dialogue.passage_groups[billboard.passage_group];

        // XXX: text/passages should not end up empty. If they are, it
        // there be a problem with the parser.
        let entire_text = &group.passages[billboard.passage];

        if billboard.head < entire_text.len() {
            let mut ui_text = data.world.write_storage::<UiText>();
            let time = data.world.read_resource::<Time>();

            let group = &dialogue.passage_groups[billboard.passage_group];

            if ui_text
                .get_mut(self.speaker_name_txt)
                .map(|t| {
                    t.text = group
                        .speaker
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("")
                        .to_string()
                })
                .is_some()
            {
                let mut since = billboard.secs_since_last_reveal.unwrap_or_default();
                since += time.delta_seconds();

                let (reveal_how_many, remainder) = calc_glyphs_to_reveal(
                    since,
                    self.glyphs_per_sec
                        * if self.fastforward {
                            TALKIE_SPEED_FACTOR
                        } else {
                            1.0
                        },
                );

                billboard.secs_since_last_reveal = Some(remainder);

                // XXX: text/passages should not end up empty. If they are, it
                // there be a problem with the parser.
                let entire_text = &group.passages[billboard.passage];

                if let Some(t) = ui_text.get_mut(self.dialogue_txt) {
                    billboard.head += reveal_how_many; // Only advance if we can update the display
                    t.text = entire_text.chars().take(billboard.head).collect();
                }
            }
            Trans::None
        } else {
            let last_group = billboard.passage_group == dialogue.passage_groups.len() - 1;
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
                return Trans::Push(Box::new(ChoiceState::new(group.choices.clone().unwrap())));
            }

            Trans::Push(Box::new(PromptState::new("confirm")))
        }
    }
}
