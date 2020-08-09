use amethyst::renderer::{Sprite, SpriteSheet, Texture};
use amethyst::{
    assets::{Format, Handle},
    error::Error,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FrameSpec {
    frame: Frame,
}

#[derive(Debug, Clone, Deserialize)]
struct Frame {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct Dimensions {
    w: u32,
    h: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct Meta {
    size: Dimensions,
}

#[derive(Debug, Clone, Deserialize)]
struct AsepriteExport {
    frames: Vec<FrameSpec>,
    meta: Meta,
}

#[derive(Debug, Clone)]
pub struct AsepriteFormat(Handle<Texture>);

impl Format<SpriteSheet> for AsepriteFormat {
    fn name(&self) -> &'static str {
        "amethyst_aseprite::AsepriteFormat"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<SpriteSheet, Error> {
        let parsed: AsepriteExport = serde_json::from_slice(&bytes)?;
        let image_w = parsed.meta.size.w;
        let image_h = parsed.meta.size.h;
        // Per the amethyst book:
        //
        //     Number of pixels to shift the sprite to the left and down relative to
        //     the entity holding it when rendering.
        //
        // Need to figure out how best to handle this, but for now hardcode to zeros.
        let offsets = [0.0; 2];

        let sprites = parsed
            .frames
            .into_iter()
            .map(|FrameSpec { frame, .. }| {
                Sprite::from_pixel_values(
                    image_w, image_h, frame.w, frame.h, frame.x, frame.y, offsets, false, false,
                )
            })
            .collect();

        Ok(SpriteSheet {
            texture: self.0.clone(),
            sprites,
        })
    }
}
