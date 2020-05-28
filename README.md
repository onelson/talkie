# talkie

This project represents, uh, prototype-grade code aimed towards implementing
some kind of dialogue tree, as you might see in old RPG games.

Broad strokes, we're talking about a box on the lower portion of the screen with
text, loaded in gradually with a prompt to load more at page boundaries.

Additionally, there may be a portrait of the "speaker" either inline with the
text, or somewhere above.

Likely we'll also want some sort of graphic (animated?) behind the text box and
portrait.

For a more exhaustive list of *"stuff we hope to build"* see: [ideas].

Ultimately, the yield from this project will come in the form of experience.
If there's to be any reusable tech, it'll be extracted as other crates.

Misc notes/docs will be accumulated under the `notes/` directory.

## Dependencies

Consult the amethyst readme for how to install the framework dependencies:
<https://github.com/amethyst/amethyst/#dependencies>

## How to run

To run the game, run the following command, which defaults to the `vulkan` graphics backend:

```bash
cargo run
```

Windows and Linux users may explicitly choose `"vulkan"` with the following command:

```bash
cargo run --no-default-features --features "vulkan"
```

Mac OS X users may explicitly choose `"metal"` with the following command:

```bash
cargo run --no-default-features --features "metal"
```

[ideas]: ./notes/ideas.md
