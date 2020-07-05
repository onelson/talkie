use crate::assets::dialogue::{Dialogue, DialogueFormat, DialogueHandle};
use crate::components::ActionTracker;
use amethyst::{
    assets::{AssetStorage, Loader, ProgressCounter},
    ecs::prelude::{Builder, WorldExt},
    prelude::{GameData, SimpleState, StateData},
    renderer::Transparent,
    ui::UiCreator,
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
    pub paused: bool,
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
            }))
        } else {
            Trans::None
        }
    }
}

/// Render the dialogue text over time.
struct PlaybackState {
    dialogue_handle: DialogueHandle,
}

impl SimpleState for PlaybackState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        world.insert(BillboardData {
            dialogue_id: self.dialogue_handle.id(),
            head: 0,
            passage_group: 0,
            passage: 0,
            paused: false,
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
        let mut billboard = data.world.write_resource::<BillboardData>();
        let dialogue_storage = data.world.read_resource::<AssetStorage<Dialogue>>();
        let dialogue = dialogue_storage
            .get_by_id(self.dialogue_handle.id())
            .unwrap();
        let group = &dialogue.passage_groups[billboard.passage_group];

        // XXX: text/passages should not end up empty. If they are, it
        // there be a problem with the parser.
        let entire_text = &group.passages[billboard.passage];

        if billboard.head < entire_text.len() {
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

            Trans::None // FIXME: transition to waiting for user input
        }
    }
}
