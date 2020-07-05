use crate::assets::dialogue::Dialogue;
use amethyst::{
    assets::Processor,
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::application_root_dir,
};

mod assets;
mod components;
mod states;
mod systems;

use states::LoadingState;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");
    let config_dir = app_root.join("config");
    let display_config_path = config_dir.join("display.ron");
    let bindings_path = config_dir.join("bindings.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.03, 0.04, 0.08, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default()),
        )?
        .with_bundle(InputBundle::<StringBindings>::new().with_bindings_from_file(bindings_path)?)?
        .with_bundle(TransformBundle::new())?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with(Processor::<Dialogue>::new(), "dialogue_processor", &[])
        .with(
            systems::ActionTrackerSystem,
            "action_tracker",
            &["input_system"],
        )
        .with(
            systems::BillboardDisplaySystem {
                glyph_speed: std::env::var("TALKIE_SPEED")
                    .map(|s| s.parse().expect("invalid speed."))
                    .ok(),
            },
            "billboard_display",
            &["dialogue_processor"],
        );

    let mut game = Application::new(
        assets_dir,
        LoadingState::new("dialogue/mgs3-body-snatchers.dialogue"),
        game_data,
    )?;
    game.run();

    Ok(())
}
