use crate::assets::dialogue::{Choice, Dialogue, DialogueFormat, DialogueHandle};
use crate::components::ActionTracker;
use crate::utils::calc_glyphs_to_reveal;
use amethyst::{
    assets::{AssetStorage, Loader, ProgressCounter},
    core::{timing::Time, transform::Transform, HiddenPropagate},
    ecs::prelude::{Builder, Entity, WorldExt},
    input::{InputHandler, StringBindings},
    prelude::{GameData, SimpleState, StateData},
    renderer::{
        camera::{Camera, Projection},
        Transparent,
    },
    ui::{UiCreator, UiFinder, UiText},
    window::ScreenDimensions,
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

/// Render the dialogue text over time.
struct PlaybackState {
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

/// The default number of glyphs to reveal per second.
///
/// This value is used as a fallback for when the `TALKIE_SPEED` env var is
/// unset while constructing a new `PlaybackState`.
const DEFAULT_GLYPHS_PER_SEC: f32 = 18.0;
const TALKIE_SPEED_FACTOR: f32 = 30.0;

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

        let billboard = world.create_entity().with(Transparent).build();
        world.insert(billboard);
    }

    fn on_resume(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        // Reset the depressed status when this state is revisited.
        self.fastforward = false;
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
                .map(|t| t.text = format!("// {}", &group.speaker))
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

struct ChoiceState {
    choices: Vec<Choice>,
}

impl ChoiceState {
    pub fn new(choices: Vec<Choice>) -> ChoiceState {
        ChoiceState { choices }
    }
}

impl SimpleState for ChoiceState {
    fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        // TODO: build a series of buttons based on `self.choices`
    }

    fn on_stop(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        // TODO: remove the buttons
    }

    fn fixed_update(&mut self, _data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        println!("{:?}", self.choices);
        Trans::Pop
    }
}

// FIXME: add a bool field for "hidden" which will drive the entity modifications over time.
//  By driving the hide/show with a separate field on the state, we can do things
//  like blink it or whatever.
struct PromptState {
    icon: Option<Entity>,
    tracker: ActionTracker,
}

impl PromptState {
    pub fn new(action: &str) -> PromptState {
        PromptState {
            icon: None,
            tracker: ActionTracker::new(action),
        }
    }
}

impl SimpleState for PromptState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        if self.icon.is_none() {
            world.exec(|ui_finder: UiFinder| {
                self.icon = ui_finder.find("next_page");
            });
        }

        let input = world.read_resource::<InputHandler<StringBindings>>();
        // By updating the tracker on start, we maintain continuity with the
        // input state of the previous state.
        // This is important because we want to *transition out of this state*
        // only when the button has been released and re-pressed.
        // Without this initial update, the player could just hold the button
        // down to advance through all passages.
        self.tracker.update(&input);
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        if let Some(icon) = self.icon {
            let mut storage = data.world.write_storage::<HiddenPropagate>();
            let _ = storage.insert(icon, HiddenPropagate::new());
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        match self.icon {
            None => return Trans::None,
            Some(icon) => {
                let mut storage = data.world.write_storage::<HiddenPropagate>();
                let _ = storage.remove(icon);
            }
        }

        let input = data.world.read_resource::<InputHandler<StringBindings>>();

        self.tracker.update(&input);

        if self.tracker.press_begin() {
            Trans::Pop
        } else {
            Trans::None
        }
    }
}
