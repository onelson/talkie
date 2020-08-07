use crate::assets::Choice;
use crate::components::ActionTracker;
use amethyst::assets::Loader;
use amethyst::core::ecs::{Builder, Entity, World, WorldExt};
use amethyst::core::HiddenPropagate;
use amethyst::input::{InputHandler, StringBindings};
use amethyst::ui::{Anchor, FontHandle, TtfFormat, UiFinder, UiText, UiTransform};
use amethyst::{GameData, SimpleState, SimpleTrans, StateData, Trans};

/// A `Resource` to track the outcomes of `ChoiceState`s.
///
/// If a button in a `ChoiceState` is activated, the value of it's `goto` field
/// will be written to this `Resource`.
#[derive(Default)]
pub struct Goto {
    /// A passage group to jump to.
    pub passage_group_id: Option<String>,
}

pub struct ChoiceState {
    choices: Vec<Choice>,
    buttons: Vec<Entity>,
    font: Option<FontHandle>,
    // y offsets for each of the buttons.
    ys: Vec<f32>,
    cursor_pos: usize,
    cursor_gfx: Option<Entity>,

    // trackers
    confirm: ActionTracker,
    up: ActionTracker,
    down: ActionTracker,
}

const GUTTER_V: f32 = 4.;
const BTN_HEIGHT: f32 = 32.;
const BTN_WIDTH: f32 = 100.;

impl SimpleState for ChoiceState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        self.poll_inputs(world);

        self.font = Some(world.read_resource::<Loader>().load(
            "font/CC Accidenz Commons-medium.ttf",
            TtfFormat,
            (),
            &world.read_resource(),
        ));

        // we don't want to capture `self` in the closure below.
        let font = self.font.clone().unwrap();

        self.buttons = self
            .ys
            .iter()
            .zip(self.choices.iter())
            .map(move |(&y, choice)| {
                let mut ui_text =
                    UiText::new(font.clone(), choice.label.clone(), [0., 0., 0., 1.], 20.0);
                ui_text.align = Anchor::MiddleLeft;

                world
                    .create_entity()
                    .with(UiTransform::new(
                        choice.label.clone(),
                        Anchor::BottomLeft,
                        Anchor::BottomLeft,
                        60.,
                        y,
                        3., // z index
                        BTN_WIDTH,
                        BTN_HEIGHT,
                    ))
                    .with(ui_text)
                    .build()
            })
            .collect();
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.delete_entities(&self.buttons).unwrap();
        self.buttons.clear();

        if let Some(gfx) = self.cursor_gfx {
            let mut storage = data.world.write_storage::<HiddenPropagate>();
            let _ = storage.insert(gfx, HiddenPropagate::new());
        }
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.confirm.press_begin() {
            log::debug!(
                "cursor => {} {:?}",
                self.cursor_pos,
                self.choices[self.cursor_pos]
            );
            let mut goto = data.world.try_fetch_mut::<Goto>().unwrap();
            goto.passage_group_id = self.choices[self.cursor_pos].goto.clone();
            return Trans::Pop;
        }
        SimpleTrans::None
    }

    fn shadow_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        if self.cursor_gfx.is_none() {
            world.exec(|ui_finder: UiFinder| {
                self.cursor_gfx = ui_finder.find("cursor");
            });
        }

        if let Some(gfx) = self.cursor_gfx {
            let mut storage = world.write_storage::<HiddenPropagate>();
            let _ = storage.remove(gfx);

            let mut storage = world.write_storage::<UiTransform>();
            let xform = storage.get_mut(gfx).unwrap();
            xform.local_y = self.ys[self.cursor_pos];
        }

        self.poll_inputs(world);
    }
}

impl ChoiceState {
    pub fn new(choices: Vec<Choice>) -> ChoiceState {
        let count = choices.len();
        let buttons = Vec::with_capacity(count);
        ChoiceState {
            choices,
            cursor_pos: 0,
            cursor_gfx: None,
            buttons,
            font: None,
            ys: (0..count)
                .map(|idx| (count - idx) as f32 * (BTN_HEIGHT + GUTTER_V))
                .collect(),

            confirm: ActionTracker::new("confirm"),
            up: ActionTracker::new("up"),
            down: ActionTracker::new("down"),
        }
    }

    fn poll_inputs(&mut self, world: &mut World) {
        let input = world.read_resource::<InputHandler<StringBindings>>();
        self.confirm.update(&input);
        self.down.update(&input);
        self.up.update(&input);

        if self.up.press_begin() {
            self.cursor_pos = if self.cursor_pos > 0 {
                self.cursor_pos - 1
            } else {
                0
            };
        } else if self.down.press_begin() {
            // choices better me non-empty...
            let max = self.choices.len() - 1;
            self.cursor_pos = if self.cursor_pos < max {
                self.cursor_pos + 1
            } else {
                max
            };
        }
    }
}
