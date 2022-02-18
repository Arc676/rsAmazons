// Copyright (C) 2022 Arc676/Alessandro Vinciguerra <alesvinciguerra@gmail.com>
// Based on public eframe template https://github.com/emilk/eframe_template

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation (version 3).

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use eframe::{egui, epi};
use eframe::egui::{Color32, emath, Painter, Pos2, Rect, Sense, Separator, Slider, Ui};
use eframe::egui::emath::RectTransform;

type PosVec = Vec<(u32, u32)>;

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct AmazonsGame {
    board_height: u32,
    board_width: u32,

    white_amazons: u32,
    black_amazons: u32,

    white_starting: PosVec,
    black_starting: PosVec,

    white_pos: PosVec,
    black_pos: PosVec,
}

impl Default for AmazonsGame {
    fn default() -> Self {
        AmazonsGame{
            board_height: 10,
            board_width: 10,
            white_amazons: 4,
            black_amazons: 4,
            white_starting: vec![
                (3, 0), (0, 3), (0, 6), (3, 9)
            ],
            black_starting: vec![
                (6, 0), (9, 3), (9, 6), (6, 9)
            ],
            white_pos: vec![],
            black_pos: vec![]
        }
    }
}

impl AmazonsGame {
    pub fn new_game(&mut self) {
        self.white_pos = self.white_starting.clone();
        self.black_pos = self.black_starting.clone();
    }

    fn draw_board(&self, painter: &Painter, to_screen: RectTransform) {
        let square_size = (1. / self.board_height as f32)
            .min(1. / self.board_width as f32);
        for x in 0..self.board_width {
            for y in 0..self.board_height {
                if (x + y) % 2 == 0 {
                    let x = x as f32 * square_size;
                    let y = y as f32 * square_size;
                    let rect = Rect{
                        min: to_screen * Pos2 { x, y },
                        max: to_screen * Pos2 { x: x + square_size, y: y + square_size }
                    };
                    painter.rect_filled(rect, 0., Color32::GRAY);
                }
            }
        }
    }
}

fn number_setting(ui: &mut Ui, num: &mut u32, min: u32, max: u32, lbl: &str) {
    let slider = Slider::new(num, min..=max).text(lbl);
    ui.add(slider);
}

impl epi::App for AmazonsGame {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Settings");

            number_setting(ui, &mut self.white_amazons, 0, 10, "Player 1 pieces");
            number_setting(ui, &mut self.black_amazons, 0, 10, "Player 2 pieces");
            number_setting(ui, &mut self.board_width, 2, 20, "Board width");
            number_setting(ui, &mut self.board_height, 2, 20, "Board height");

            if ui.button("Set player 1 starting positions").clicked() {
            }

            if ui.button("Set player 2 starting positions").clicked() {
            }

            if ui.button("Revert to default parameters").clicked() {
                *self = AmazonsGame::default();
            }

            let sep = Separator::default().spacing(12.).horizontal();
            ui.add(sep);

            if ui.button("Quit").clicked() {
                frame.quit();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Game of the Amazons");
            let (response, painter) = ui.allocate_painter(
                ui.available_size_before_wrap(), Sense::click());
            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
                response.rect,
            );
            self.draw_board(&painter, to_screen);
        });
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn name(&self) -> &str {
        "Game of the Amazons"
    }
}
