use crate::assets::dialogue::{Dialogue, DialogueFormat, DialogueHandle};
use crate::components::ActionTracker;
use amethyst::{
    assets::{AssetStorage, Loader, ProgressCounter},
    core::timing::Time,
    ecs::prelude::{Builder, Entity, WorldExt},
    input::{InputHandler, StringBindings},
    prelude::{GameData, SimpleState, StateData},
    renderer::Transparent,
    ui::{UiCreator, UiFinder, UiText},
    SimpleTrans, Trans,
};

/// The "billboard" is the lower half of the window where dialogue text will
/// appear.
#[derive(Default)]
pub struct BillboardData {
    pub dialogue_id: u32,
    /// tracks the current length of *displayed text*.
    pub head: usize,
    /// tracks which passage group we're iterating through.
    pub passage_group: usize,
    /// tracks which passage we're showing.
    pub passage: usize,
    /// tracks the time since the last glyph was revealed.
    // XXX: We could default this to 0.0 and not bother with the Option, but
    //  I thought it might be interesting to be able to know when we're starting
    //  totally from scratch vs rolling over from a previous iteration.
    pub secs_since_last_reveal: Option<f32>,
}

/// Load up the dialogue from disk.
pub struct LoadingState {
    path: String,
    dialogue_handle: Option<DialogueHandle>,
    progress_counter: ProgressCounter,
}

impl LoadingState {
    pub fn new<P: Into<String>>(dialogue_path: P) -> LoadingState {
        LoadingState {
            path: dialogue_path.into(),
            dialogue_handle: None,
            progress_counter: ProgressCounter::new(),
        }
    }
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        self.dialogue_handle = Some(world.read_resource::<Loader>().load(
            &self.path,
            DialogueFormat,
            &mut self.progress_counter,
            &world.read_resource(),
        ));
    }

    fn fixed_update(&mut self, _data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.progress_counter.is_complete() {
            Trans::Switch(Box::new(PlaybackState {
                dialogue_handle: self.dialogue_handle.take().unwrap(),
                glyph_speed: std::env::var("TALKIE_SPEED")
                    .map(|s| s.parse().expect("invalid speed."))
                    .ok(),
                speaker_name_txt: None,
                dialogue_txt: None,
            }))
        } else {
            Trans::None
        }
    }
}

/// Render the dialogue text over time.
struct PlaybackState {
    dialogue_handle: DialogueHandle,
    glyph_speed: Option<f32>,
    speaker_name_txt: Option<Entity>,
    dialogue_txt: Option<Entity>,
}

/// A new glyph is revealed when this amount of time has passed.
const GLYPH_PERIOD_SECS: f32 = 0.2;

impl SimpleState for PlaybackState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.insert(BillboardData {
            dialogue_id: self.dialogue_handle.id(),
            head: 0,
            passage_group: 0,
            passage: 0,
            secs_since_last_reveal: None,
        });
        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("billboard.ron", ());
        });
        let billboard = world
            .create_entity()
            .with(Transparent)
            .with(ActionTracker::new("confirm"))
            .build();
        world.insert(billboard);
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.speaker_name_txt.or(self.dialogue_txt).is_none() {
            // bail early if we haven't loaded the ui yet.
            return Trans::None;
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

            if self
                .speaker_name_txt
                .and_then(|e| ui_text.get_mut(e))
                .map(|t| t.text = format!("// {}", &group.speaker))
                .is_some()
            {
                let mut since = billboard.secs_since_last_reveal.unwrap_or_default();
                since += time.delta_seconds();

                let glyph_speed = self.glyph_speed.unwrap_or(GLYPH_PERIOD_SECS);
                let reveal_how_many = (since / glyph_speed).trunc() as usize;
                let remainder = since % glyph_speed;

                billboard.secs_since_last_reveal = Some(remainder);

                // XXX: text/passages should not end up empty. If they are, it
                // there be a problem with the parser.
                let entire_text = &group.passages[billboard.passage];

                if let Some(entity) = self.dialogue_txt {
                    if let Some(t) = ui_text.get_mut(entity) {
                        billboard.head += reveal_how_many; // Only advance if we can update the display
                        t.text = entire_text.chars().take(billboard.head).collect();
                    }
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

            Trans::Push(Box::new(PromptState::new("confirm")))
        }
    }

    fn shadow_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // FIXME: think about moving this into the loading state
        //  Will need to figure out how to hide the UI elements until "on_start" fires in playback
        //  to do this.
        if self.speaker_name_txt.or(self.dialogue_txt).is_none() {
            data.world.exec(|ui_finder: UiFinder| {
                self.speaker_name_txt = ui_finder.find("speaker_name");
                self.dialogue_txt = ui_finder.find("dialogue_text");
            });
        }
    }
}

#[derive(Default)]
struct SleepState {
    /// How many seconds to wait before "popping" to return to the previous state.
    sleep_duration: f32,
    /// How long this state has slept for.
    acc: f32,
}

impl SleepState {
    pub fn new(sleep_duration: f32) -> SleepState {
        SleepState {
            sleep_duration,
            ..Default::default()
        }
    }
}

impl SimpleState for SleepState {
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.acc < self.sleep_duration {
            let time = data.world.read_resource::<Time>();
            self.acc += time.delta_seconds();
            Trans::None
        } else {
            Trans::Pop
        }
    }
}

struct PromptState {
    tracker: ActionTracker,
}

impl PromptState {
    pub fn new(action: &str) -> PromptState {
        PromptState {
            tracker: ActionTracker::new(action),
        }
    }
}

impl SimpleState for PromptState {
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let input = data.world.read_resource::<InputHandler<StringBindings>>();
        self.tracker.update(&input);
        if self.tracker.press_begin() {
            Trans::Pop
        } else {
            Trans::None
        }
    }
}
