// Taken from public eframe template (with light modifications)
// https://github.com/emilk/eframe_template

mod app;
mod boardstate;
mod sprites;

use app::AmazonsGame;

fn main() {
    let app = AmazonsGame::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
