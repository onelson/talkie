use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::ecs::{Builder, Entity, World, WorldExt};
use amethyst::core::Hidden;
use amethyst::renderer::{ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture};
use amethyst::ui::{Anchor, UiImage, UiTransform};

fn load_texture<N>(name: N, world: &World) -> Handle<Texture>
where
    N: Into<String>,
{
    let loader = world.read_resource::<Loader>();
    loader.load(
        name,
        ImageFormat::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    )
}

fn load_sprite_sheet<N>(name: N, tex: Handle<Texture>, world: &World) -> Handle<SpriteSheet>
where
    N: Into<String>,
{
    let loader = world.read_resource::<Loader>();
    let storage = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(name, SpriteSheetFormat(tex), (), &storage)
}

pub fn build_sprite_sheet(world: &World) -> Handle<SpriteSheet> {
    let tex = load_texture("img/icons.png", world);
    load_sprite_sheet("icons.ron", tex, world)
}

fn ui_image_from_sprite(sprite_number: usize, sprite_sheet: Handle<SpriteSheet>) -> UiImage {
    let sr = SpriteRender {
        sprite_sheet,
        sprite_number,
    };
    UiImage::Sprite(sr)
}

const SPRITE_NEXT_PAGE: usize = 0;
const SPRITE_CURSOR: usize = 1;

fn build_next_page(sprite_sheet: Handle<SpriteSheet>, world: &mut World) -> Entity {
    let xform = UiTransform::new(
        String::from("next_page"),
        Anchor::BottomLeft,
        Anchor::BottomLeft,
        448.0,
        6.0,
        2.0,
        32.0,
        32.0,
    );

    world
        .create_entity()
        .with(xform)
        .with(ui_image_from_sprite(SPRITE_NEXT_PAGE, sprite_sheet))
        .with(Hidden)
        .build()
}

fn build_cursor(sprite_sheet: Handle<SpriteSheet>, world: &mut World) -> Entity {
    let xform = UiTransform::new(
        String::from("cursor"),
        Anchor::BottomLeft,
        Anchor::BottomLeft,
        30.0,
        0.0,
        2.0,
        32.0,
        32.0,
    );
    world
        .create_entity()
        .with(xform)
        .with(ui_image_from_sprite(SPRITE_CURSOR, sprite_sheet))
        .with(Hidden)
        .build()
}

pub fn build_icon_entities(sprite_sheet: Handle<SpriteSheet>, world: &mut World) {
    build_next_page(sprite_sheet.clone(), world);
    build_cursor(sprite_sheet, world);
}
