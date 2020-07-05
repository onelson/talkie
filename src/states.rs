use crate::assets::dialogue::{DialogueFormat, DialogueHandle};
use crate::components::{ActionTracker, BillboardData};
use amethyst::{
    assets::{Loader, ProgressCounter},
    ecs::prelude::{Builder, WorldExt},
    prelude::{GameData, SimpleState, StateData},
    renderer::Transparent,
    ui::UiCreator,
    SimpleTrans, Trans,
};

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

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("billboard.ron", ());
        });

        let billboard = world
            .create_entity()
            .with(Transparent)
            .with(BillboardData {
                dialogue: self.dialogue_handle.clone(),
                head: 0,
                passage_group: 0,
                passage: 0,
                paused: false,
                secs_since_last_reveal: None,
            })
            .with(ActionTracker::new("confirm"))
            .build();

        world.insert(billboard);
    }

    fn fixed_update(&mut self, _data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        Trans::None
    }
}
