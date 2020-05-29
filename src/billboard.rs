//! The "billboard" is the lower half of the window where dialogue text will
//! appear.

use amethyst::assets::Loader;
use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};
use amethyst::prelude::*;
use amethyst::ui::{Anchor, LineMode, TtfFormat, UiText, UiTransform};

#[derive(Default)]
pub struct Billboard {}

pub struct BillboardText {
    pub text: Entity,
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

    let mut text = UiText::new(font.clone(), LIPSUM.to_string(), [1., 1., 1., 1.], 24.);
    text.line_mode = LineMode::Wrap;
    text.align = Anchor::MiddleLeft;

    let text_entity = world.create_entity().with(xform).with(text).build();

    world.insert(BillboardText { text: text_entity });
}
