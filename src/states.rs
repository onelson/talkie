use crate::assets::dialogue::{Dialogue, DialogueFormat, DialogueHandle};
use crate::components::ActionTracker;
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
    }

    fn fixed_update(&mut self, _data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.progress_counter.is_complete() {
            Trans::Switch(Box::new(PlaybackState::new(
                self.dialogue_handle.take().unwrap(),
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
    speaker_name_txt: Option<Entity>,
    dialogue_txt: Option<Entity>,
    /// Tracks if the confirm action was ever "not pressed."
    /// This is useful for preventing the text from speeding up immediately
    /// after popping the `PromptState`.
    depressed: bool,
}

impl PlaybackState {
    pub fn new(dialogue_handle: DialogueHandle) -> PlaybackState {
        PlaybackState {
            dialogue_handle,
            glyphs_per_sec: std::env::var("TALKIE_SPEED")
                .map(|s| s.parse().expect("invalid speed."))
                .unwrap_or(DEFAULT_GLYPHS_PER_SEC),
            speaker_name_txt: None,
            dialogue_txt: None,
            tracker: ActionTracker::new("confirm"),
            depressed: false,
        }
    }
}

/// The default number of glyphs to reveal per second.
///
/// This value is used as a fallback for when the `TALKIE_SPEED` env var is
/// unset while constructing a new `PlaybackState`.
const DEFAULT_GLYPHS_PER_SEC: f32 = 18.0;
const TALKIE_SPEED_FACTOR: f32 = 30.0;

/// Given some amount of time, use the rate to determine how much of the time
/// went unused and how many glyphs should now be revealed.
fn calc_glyphs_to_reveal(delta_secs: f32, glyphs_per_sec: f32) -> (usize, f32) {
    let reveal_how_many = (delta_secs * glyphs_per_sec).trunc();
    let remainder = delta_secs - (reveal_how_many / glyphs_per_sec);
    (reveal_how_many as usize, remainder)
}

#[cfg(test)]
mod glyph_reveal_tests {
    use super::calc_glyphs_to_reveal;
    use assert_approx_eq::assert_approx_eq;

    /// If the delta is not big enough to reveal at least one glyph, then the
    /// remainder should be the entire delta.
    #[test]
    fn test_delta_carries_over() {
        let (count, remainder) = calc_glyphs_to_reveal(1.0, 0.5);
        assert_eq!(0, count);
        assert_approx_eq!(1.0, remainder);
    }

    #[test]
    fn test_delta_zero_when_glyph_revealed() {
        let (count, remainder) = calc_glyphs_to_reveal(2.0, 0.5);
        assert_eq!(1, count);
        assert_approx_eq!(0.0, remainder);
    }

    #[test]
    fn test_delta_remainder_when_glyph_revealed() {
        let (count, remainder) = calc_glyphs_to_reveal(2.2, 0.5);
        assert_eq!(1, count);
        assert_approx_eq!(0.2, remainder);
    }

    #[test]
    fn test_multi_glyph_remainder() {
        let (count, remainder) = calc_glyphs_to_reveal(5.2, 2.0);
        assert_eq!(10, count);
        assert_approx_eq!(0.2, remainder);
    }
}

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
        let billboard = world.create_entity().with(Transparent).build();
        world.insert(billboard);
    }

    fn on_resume(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        // Reset the depressed status when this state is revisited.
        self.depressed = false;
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let input = data.world.read_resource::<InputHandler<StringBindings>>();
        self.tracker.update(&input);

        if !self.depressed && !self.tracker.pressed() {
            self.depressed = true;
        }

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

                // We expect the action to be released and re-pressed before it
                // speeds up the text. This guards against the `PromptState`
                // confirm from fast-forwarding the text that immediately starts
                // showing after the pop.
                let should_speed_up = self.depressed && self.tracker.pressed();
                let (reveal_how_many, remainder) = calc_glyphs_to_reveal(
                    since,
                    self.glyphs_per_sec
                        * if should_speed_up {
                            TALKIE_SPEED_FACTOR
                        } else {
                            1.0
                        },
                );

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

// FIXME: add a bool field for "hidden" which will drive the entity modifications over time.
//  By driving the hide/show with a separate field on the state, we can do things
//  like blink it or whatever.
struct PromptState {
    icon: Option<Entity>,
    tracker: ActionTracker,
    depressed: bool,
    pressed: bool,
}

impl PromptState {
    pub fn new(action: &str) -> PromptState {
        PromptState {
            icon: None,
            tracker: ActionTracker::new(action),
            depressed: false,
            pressed: false,
        }
    }
}

impl SimpleState for PromptState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let input = data.world.read_resource::<InputHandler<StringBindings>>();
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
        if !self.depressed && !self.tracker.pressed() {
            self.depressed = true;
        }

        if self.depressed && !self.pressed && self.tracker.pressed() {
            self.pressed = true;
        }

        if self.depressed && self.pressed {
            Trans::Pop
        } else {
            Trans::None
        }
    }

    fn shadow_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        if self.icon.is_none() {
            data.world.exec(|ui_finder: UiFinder| {
                self.icon = ui_finder.find("next_page");
            });
        }
    }
}
