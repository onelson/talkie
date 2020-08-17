use amethyst::{
    assets::{Asset, Format, Handle},
    ecs::HashMapStorage,
    error::Error,
};

pub use omn_sprites::ClipStore as ClipStore_;
use omn_sprites::SpriteSheetData;

pub struct ClipStore(pub ClipStore_);
pub type ClipStoreHandle = Handle<ClipStore>;

impl Asset for ClipStore {
    const NAME: &'static str = "ClipStore";
    type Data = Self;
    type HandleStorage = HashMapStorage<ClipStoreHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ClipStoreFormat;

impl Format<ClipStore> for ClipStoreFormat {
    fn name(&self) -> &'static str {
        "ClipStoreFormat"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<ClipStore, Error> {
        // FIXME: omn_sprites should update to return a result from this!
        let data = SpriteSheetData::from_json_value(serde_json::from_slice(&bytes)?);
        Ok(ClipStore(ClipStore_::new(&data)))
    }
}
