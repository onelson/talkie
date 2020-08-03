use crate::assets::dialogue::{DialogueFormat, DialogueHandle};
use crate::states::playback::PlaybackState;
use amethyst::assets::{Loader, ProgressCounter};
use amethyst::core::ecs::{Builder, WorldExt};
use amethyst::core::Transform;
use amethyst::renderer::camera::Projection;
use amethyst::renderer::Camera;
use amethyst::ui::{UiCreator, UiFinder};
use amethyst::window::ScreenDimensions;
use amethyst::{GameData, SimpleState, SimpleTrans, StateData, Trans};

mod choice;
mod playback;
mod prompt;

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
    /// Tracks the time since the last glyph was revealed.
    // XXX: We could default this to 0.0 and not bother with the Option, but
    //  I thought it might be interesting to be able to know when we're starting
    //  totally from scratch vs rolling over from a previous iteration.
    pub secs_since_last_reveal: Option<f32>,
}

/// Load up the dialogue from disk.
pub struct LoadingState {
    path: String,
    dialogue_handle: Option<DialogueHandle>,
    dialogue_progress: ProgressCounter,
    ui_progress: ProgressCounter,
}

impl LoadingState {
    pub fn new<P: Into<String>>(dialogue_path: P) -> LoadingState {
        LoadingState {
            path: dialogue_path.into(),
            dialogue_handle: None,
            dialogue_progress: ProgressCounter::new(),
            ui_progress: ProgressCounter::new(),
        }
    }
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        self.dialogue_handle = Some(world.read_resource::<Loader>().load(
            &self.path,
            DialogueFormat,
            &mut self.dialogue_progress,
            &world.read_resource(),
        ));

        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        // Translate the camera to Z coordinate 10.0, and it looks back toward
        // the origin with depth 20.0
        let mut transform = Transform::default();
        transform.set_translation_xyz(0., 0., 10.);

        let mut camera = Camera::standard_3d(width, height);
        camera.set_projection(Projection::orthographic(0., width, 0., height, 0.0, 20.0));

        let _camera = world.create_entity().with(transform).with(camera).build();

        world.exec(move |mut creator: UiCreator<'_>| {
            creator.create("billboard.ron", &mut self.ui_progress);
        });
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.dialogue_progress.is_complete() && self.ui_progress.is_complete() {
            let mut speaker_name_txt = None;
            let mut dialogue_txt = None;

            data.world.exec(|ui_finder: UiFinder| {
                speaker_name_txt = ui_finder.find("speaker_name");
                dialogue_txt = ui_finder.find("dialogue_text");
            });

            Trans::Switch(Box::new(PlaybackState::new(
                self.dialogue_handle.take().unwrap(),
                speaker_name_txt.unwrap(),
                dialogue_txt.unwrap(),
            )))
        } else {
            Trans::None
        }
    }
}
