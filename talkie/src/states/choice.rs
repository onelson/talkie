use crate::assets::Choice;
use amethyst::assets::Loader;
use amethyst::core::ecs::{Builder, Entity, WorldExt};
use amethyst::input::StringBindings;
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
}

impl SimpleState for ChoiceState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        self.font = Some(world.read_resource::<Loader>().load(
            "font/CC Accidenz Commons-medium.ttf",
            TtfFormat,
            (),
            &world.read_resource(),
        ));

        // we don't want to capture `self` in the closure below.
        let font = self.font.clone().unwrap();

        self.buttons = self
            .choices
            .iter()
            // rev to make it so that the idx is lower for the later choices
            // Since y goes "up" based on the bottom left anchors, we want the
            // earlier items to have higher values than the later ones for the
            // purpose of generating y offsets.
            .rev()
            .enumerate()
            .map(move |(idx, choice)| {
                let mut ui_text =
                    UiText::new(font.clone(), choice.label.clone(), [0., 0., 0., 1.], 20.0);
                ui_text.align = Anchor::MiddleLeft;

                const GUTTER_V: f32 = 4.;
                const BTN_HEIGHT: f32 = 30.;
                const BTN_WIDTH: f32 = 100.;
                // Mainly used to make room for the billboard bezel.
                const PADDING: f32 = 30.;

                world
                    .create_entity()
                    .with(UiTransform::new(
                        choice.label.clone(),
                        Anchor::BottomLeft,
                        Anchor::BottomLeft,
                        PADDING,
                        (idx as f32 * (BTN_HEIGHT + GUTTER_V)) + PADDING,
                        3., // z index
                        BTN_WIDTH,
                        BTN_HEIGHT,
                    ))
                    .with(ui_text)
                    .with(Interactable)
                    .build()
            })
            // rev again so that the button index lines up with the choice index.
            .rev()
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
        let buttons = Vec::with_capacity(choices.len());
        ChoiceState {
            choices,
            buttons,
            font: None,
        }
    }
}
