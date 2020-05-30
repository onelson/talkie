//! The "billboard" is the lower half of the window where dialogue text will
//! appear.

use amethyst::{
    assets::Loader,
    derive::SystemDesc,
    ecs::{
        prelude::{Component, HashMapStorage},
        Join, System, SystemData, World, WriteStorage,
    },
    prelude::*,
    ui::{Anchor, LineMode, TtfFormat, UiText, UiTransform},
};

#[derive(Default)]
pub struct BillboardData {
    /// the full text to display
    pub entire_text: String,
    /// tracks the current length of *displayed text*.
    pub head: usize,
}

impl Component for BillboardData {
    type Storage = HashMapStorage<Self>;
}

static LIPSUM: &str = "\
Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
Duis venenatis rutrum sapien, eu vehicula sapien cursus in. \
Donec vel pulvinar turpis. Aliquam vestibulum gravida nibh interdum malesuada. \
Sed rutrum orci nec quam aliquam, id consectetur lorem vestibulum. \
Praesent cursus justo in orci finibus, non porttitor eros pellentesque. \
Sed bibendum lectus.\
";

pub fn init_billboard(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/CC Accidenz Commons-medium.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );

    let xform = UiTransform::new(
        "text".to_string(),
        Anchor::BottomLeft,
        Anchor::BottomLeft,
        5.,
        5.,
        1.,
        // based on a 500x500 window
        490.,
        250.,
    );

    let mut ui_text = UiText::new(font.clone(), String::new(), [1., 1., 1., 1.], 24.);
    ui_text.line_mode = LineMode::Wrap;
    ui_text.align = Anchor::TopLeft;

    let billboard = world
        .create_entity()
        .with(xform)
        .with(ui_text)
        .with(BillboardData {
            entire_text: LIPSUM.to_string(),
            head: 0,
        })
        .build();

    world.insert(billboard);
}

/// Updates the display of the billboard text.
#[derive(SystemDesc)]
pub struct BillboardDisplaySystem;

impl<'s> System<'s> for BillboardDisplaySystem {
    type SystemData = (WriteStorage<'s, UiText>, WriteStorage<'s, BillboardData>);

    fn run(&mut self, (mut ui_text, mut billboard): Self::SystemData) {
        // TODO write out one glyph per <unit of time> instead of per tick.
        for (text, billboard) in (&mut ui_text, &mut billboard).join() {
            text.text = billboard.entire_text.chars().take(billboard.head).collect();
            billboard.head = if billboard.head == billboard.entire_text.len() - 1 {
                0
            } else {
                billboard.head + 1
            };
        }
    }
}
