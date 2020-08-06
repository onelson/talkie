use crate::assets::Choice;
use crate::components::ActionTracker;
use amethyst::assets::Loader;
use amethyst::core::ecs::{Builder, Entity, World, WorldExt};
use amethyst::input::{InputHandler, StringBindings};
use amethyst::ui::{Anchor, FontHandle, Interactable, TtfFormat, UiEventType, UiText, UiTransform};
use amethyst::{GameData, SimpleState, SimpleTrans, StateData, StateEvent, Trans};

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

    // trackers
    confirm: ActionTracker,
    up: ActionTracker,
    down: ActionTracker,
}

const GUTTER_V: f32 = 4.;
const BTN_HEIGHT: f32 = 30.;
const BTN_WIDTH: f32 = 100.;
// Mainly used to make room for the billboard bezel.
const PADDING: f32 = 30.;

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
                        PADDING,
                        y,
                        3., // z index
                        BTN_WIDTH,
                        BTN_HEIGHT,
                    ))
                    .with(ui_text)
                    .with(Interactable)
                    .build()
            })
            .collect();
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.delete_entities(&self.buttons).unwrap();
        self.buttons.clear();
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent<StringBindings>,
    ) -> SimpleTrans {
        self.poll_inputs(data.world);
        log::info!(
            "cursor => {} {:?}",
            self.cursor_pos,
            self.choices[self.cursor_pos]
        );
        if let StateEvent::Ui(ui_event) = event {
            match ui_event.event_type {
                UiEventType::Click => {
                    if let Some(btn_idx) = self.buttons.iter().position(|e| e == &ui_event.target) {
                        let mut goto = data.world.try_fetch_mut::<Goto>().unwrap();
                        goto.passage_group_id = self.choices[btn_idx].goto.clone();
                        return Trans::Pop;
                    }
                }
                _ => {
                    return SimpleTrans::None;
                }
            };
        }
        SimpleTrans::None
    }
}

impl ChoiceState {
    pub fn new(choices: Vec<Choice>) -> ChoiceState {
        let count = choices.len();
        let buttons = Vec::with_capacity(count);
        ChoiceState {
            choices,
            cursor_pos: 0,
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
