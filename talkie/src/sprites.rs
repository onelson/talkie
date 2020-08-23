use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::ecs::{Builder, Entity, World, WorldExt};
use amethyst::core::Hidden;
use amethyst::renderer::{ImageFormat, SpriteRender, SpriteSheet, Texture, Transparent};
use amethyst::ui::{Anchor, UiImage, UiTransform};
use amethyst_aseprite::AsepriteFormat;
use omn_sprites_amethyst_ext::{ClipStore, ClipStoreFormat, ClipStoreHandle};

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
    loader.load(name, AsepriteFormat(tex), (), &storage)
}

pub fn load_clip_store<N>(name: N, world: &World) -> ClipStoreHandle
where
    N: Into<String>,
{
    let loader = world.read_resource::<Loader>();
    let storage = world.read_resource::<AssetStorage<ClipStore>>();
    loader.load(name, ClipStoreFormat, (), &storage)
}

pub fn build_sprite_sheet<N>(tex: N, data: N, world: &World) -> Handle<SpriteSheet>
where
    N: Into<String>,
{
    let tex = load_texture(tex, world);
    load_sprite_sheet(data, tex, world)
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

pub fn build_slime_entity(sprite_sheet: Handle<SpriteSheet>, world: &mut World) -> Entity {
    let xform = UiTransform::new(
        String::from("slime"),
        Anchor::BottomMiddle,
        Anchor::BottomMiddle,
        0.0,
        300.0,
        1.0,
        128.0,
        128.0,
    );
    world
        .create_entity()
        .with(xform)
        .with(ui_image_from_sprite(16, sprite_sheet))
        .with(Transparent)
        .build()
}
