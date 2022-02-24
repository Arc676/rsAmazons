# Game of the Amazons

A graphical frontend to the [Amazons library](https://github.com/Arc676/amazons) using [`egui`](https://github.com/emilk/egui/) written in Rust.

## Compilation

The repository includes the Amazons library as a submodule. The library can be compiled using the included `Makefile`. The build script adds the submodule directory as a search path for libraries to link.

Rust bindings for the C library generated using [`rust-bindgen`](https://github.com/rust-lang/rust-bindgen).

## Licensing

Project available under GPLv3. The `egui` crate is available under Apache 2.0 or MIT. This project includes code from the [`eframe` public template](https://github.com/emilk/eframe_template), which has no license. Code taken from this repository includes a notice at the top of the source file.

Sprites available under CC BY-NC-SA 4.0, adapted from [CC0 assets by rcorre](https://opengameart.org/content/rpg-itemterraincharacter-sprites-ice-insignia). See `CREDITS` for more details.
