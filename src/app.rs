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

use crate::boardstate::Amazons::*;
use crate::sprites::*;
use eframe::egui::emath::RectTransform;
use eframe::egui::{
    emath, Color32, Painter, Pos2, Rect, Sense, Separator, Slider, TextureId, Ui, Vec2,
};
use eframe::epi::Frame;
use eframe::{egui, epi};

type PosVec = Vec<(u32, u32)>;
type ImageData = (TextureId, Vec2);

#[derive(PartialEq)]
enum ClickableState {
    GameInProgress,
    PickingWhite,
    PickingBlack,
    GameOver(SquareState),
    InvalidConfig(String),
    Idle,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct AmazonsGame {
    // Board dimensions
    board_height: u32,
    board_width: u32,

    // Starting pieces
    white_amazons: u32,
    black_amazons: u32,

    // Starting positions
    white_starting: PosVec,
    black_starting: PosVec,

    // Sprites
    #[cfg_attr(feature = "persistence", serde(skip))]
    white_sprite: Option<ImageData>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    black_sprite: Option<ImageData>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    arrow_sprite: Option<ImageData>,

    // Game state
    #[cfg_attr(feature = "persistence", serde(skip))]
    state: ClickableState,
    #[cfg_attr(feature = "persistence", serde(skip))]
    boardstate: BoardState,
    #[cfg_attr(feature = "persistence", serde(skip))]
    white_squares: u32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    black_squares: u32,

    #[cfg_attr(feature = "persistence", serde(skip))]
    highlight_regions: bool,

    // User move
    #[cfg_attr(feature = "persistence", serde(skip))]
    src_square: Square,
    #[cfg_attr(feature = "persistence", serde(skip))]
    dst_square: Square,
    #[cfg_attr(feature = "persistence", serde(skip))]
    shot_square: Square,
    #[cfg_attr(feature = "persistence", serde(skip))]
    clicked_square: u8,
}

impl Default for AmazonsGame {
    fn default() -> Self {
        AmazonsGame {
            board_height: 10,
            board_width: 10,
            white_amazons: 4,
            black_amazons: 4,
            white_starting: vec![],
            black_starting: vec![],
            white_sprite: None,
            black_sprite: None,
            arrow_sprite: None,
            state: ClickableState::Idle,
            boardstate: BoardState::default(),
            white_squares: 0,
            black_squares: 0,
            highlight_regions: false,
            src_square: Square::default(),
            dst_square: Square::default(),
            shot_square: Square::default(),
            clicked_square: 0,
        }
    }
}

impl AmazonsGame {
    fn is_empty_config(&self) -> bool {
        self.white_starting.is_empty()
            && self.white_amazons == 4
            && self.black_starting.is_empty()
            && self.black_amazons == 4
            && self.board_width == 10
            && self.board_height == 10
    }

    fn config_is_valid(&self) -> Result<(), &str> {
        if self.is_empty_config() {
            return Ok(());
        }
        if self.white_starting.len() != self.white_amazons as usize {
            return Err("Starting positions for player 1 not provided");
        }
        if self.black_starting.len() != self.black_amazons as usize {
            return Err("Starting positions for player 2 not provided");
        }
        for (x, y) in &self.white_starting {
            if *x >= self.board_width || *y >= self.board_height {
                return Err("Player 1 has one or more starting positions out of bounds");
            }
            if self.black_starting.contains(&(*x, *y)) {
                return Err("Overlapping starting positions");
            }
        }
        for (x, y) in &self.black_starting {
            if *x >= self.board_width || *y >= self.board_height {
                return Err("Player 2 has one or more starting positions out of bounds");
            }
        }
        Ok(())
    }

    pub fn new_game(&mut self) {
        if let Err(e) = self.config_is_valid() {
            self.state = ClickableState::InvalidConfig(e.to_string());
            return;
        }
        if self.white_starting.is_empty() {
            unsafe {
                boardstate_standard(&mut self.boardstate);
            }
        } else {
            self.boardstate.init(
                self.white_amazons,
                self.black_amazons,
                self.board_width,
                self.board_height,
                &self.white_starting,
                &self.black_starting,
            );
        }
        self.state = ClickableState::GameInProgress;
    }

    fn load_sprites(&mut self, frame: &Frame) {
        let bows = include_bytes!("../sprites/P1.png");
        let bows = load_image_from_bytes(bows, frame);
        self.white_sprite = Some(bows);

        let spears = include_bytes!("../sprites/P2.png");
        let spears = load_image_from_bytes(spears, frame);
        self.black_sprite = Some(spears);

        let arrows = include_bytes!("../sprites/Occupied.png");
        let arrows = load_image_from_bytes(arrows, frame);
        self.arrow_sprite = Some(arrows);
    }

    fn square_size(&self) -> f32 {
        (1. / self.board_height as f32).min(1. / self.board_width as f32)
    }

    fn square_from_coords(&self, x: u32, y: u32, to_screen: RectTransform) -> Rect {
        let square_size = self.square_size();
        let x = x as f32 * square_size;
        let y = y as f32 * square_size;
        Rect {
            min: to_screen * Pos2 { x, y },
            max: to_screen
                * Pos2 {
                    x: x + square_size,
                    y: y + square_size,
                },
        }
    }

    fn draw_sprite(rect: Rect, sprite: Option<ImageData>, painter: &Painter) {
        let id = sprite.unwrap().0;
        let mut mesh = egui::epaint::Mesh::with_texture(id);
        mesh.add_rect_with_uv(
            rect,
            Rect::from_min_max(Pos2 { x: 0., y: 0. }, Pos2 { x: 1., y: 1. }),
            Color32::WHITE,
        );
        painter.add(egui::Shape::mesh(mesh));
    }

    #[allow(non_upper_case_globals)]
    fn draw_board(&mut self, painter: &Painter, to_screen: RectTransform, frame: &Frame) {
        if self.white_sprite.is_none() {
            self.load_sprites(frame)
        }
        for x in 0..self.board_width {
            for y in 0..self.board_height {
                if (x + y) % 2 == 0 {
                    let rect = self.square_from_coords(x, y, to_screen);
                    painter.rect_filled(rect, 0., Color32::GRAY);
                }
            }
        }
        match self.state {
            ClickableState::GameInProgress | ClickableState::GameOver(_) => {
                for x in 0..self.board_width {
                    for y in 0..self.board_height {
                        let rect = self.square_from_coords(x, y, to_screen);
                        let mut sq = Square::new(x, y);
                        unsafe {
                            match boardstate_squareState(&mut self.boardstate, &mut sq) {
                                SquareState_WHITE => {
                                    AmazonsGame::draw_sprite(rect, self.white_sprite, painter)
                                }
                                SquareState_BLACK => {
                                    AmazonsGame::draw_sprite(rect, self.black_sprite, painter)
                                }
                                SquareState_ARROW => {
                                    AmazonsGame::draw_sprite(rect, self.arrow_sprite, painter)
                                }
                                _ => (),
                            }
                        }
                    }
                }
                if self.clicked_square == 2 {
                    let (x, y) = self.dst_square.destructure();
                    let rect = self.square_from_coords(x, y, to_screen);
                    painter.rect_filled(rect, 0., Color32::RED);
                }
                if self.clicked_square >= 1 {
                    let (x, y) = self.src_square.destructure();
                    let rect = self.square_from_coords(x, y, to_screen);
                    painter.rect_filled(rect, 0., Color32::from_rgba_unmultiplied(0, 255, 0, 128))
                }
                if let ClickableState::GameOver(_) = self.state {
                    if self.highlight_regions {
                        for x in 0..self.board_width {
                            for y in 0..self.board_height {
                                let rect = self.square_from_coords(x, y, to_screen);
                                let mut sq = Square::new(x, y);
                                unsafe {
                                    match boardstate_squareController(&mut self.boardstate, &mut sq)
                                    {
                                        SquareState_WHITE => painter.rect_filled(
                                            rect,
                                            0.,
                                            Color32::from_rgba_unmultiplied(255, 0, 0, 128),
                                        ),
                                        SquareState_BLACK => painter.rect_filled(
                                            rect,
                                            0.,
                                            Color32::from_rgba_unmultiplied(0, 0, 255, 128),
                                        ),
                                        _ => (),
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                for (x, y) in &self.white_starting {
                    let rect = self.square_from_coords(*x, *y, to_screen);
                    AmazonsGame::draw_sprite(rect, self.white_sprite, painter);
                }
                for (x, y) in &self.black_starting {
                    let rect = self.square_from_coords(*x, *y, to_screen);
                    AmazonsGame::draw_sprite(rect, self.black_sprite, painter);
                }
            }
        }
    }

    fn set_src(&mut self, x: u32, y: u32) -> bool {
        self.src_square = Square::new(x, y);
        unsafe {
            boardstate_squareState(&mut self.boardstate, &mut self.src_square)
                == self.boardstate.currentPlayer
        }
    }

    fn set_dst(&mut self, x: u32, y: u32) -> bool {
        let mut dst = Square::new(x, y);
        unsafe {
            if isValidMove(&mut self.boardstate, &mut self.src_square, &mut dst) == 1 {
                self.dst_square = dst;
                return true;
            }
        }
        false
    }

    fn move_amazon(&mut self, x: u32, y: u32) -> bool {
        let mut shot = Square::new(x, y);
        unsafe {
            if boardstate_squareState(&mut self.boardstate, &mut self.src_square)
                == self.boardstate.currentPlayer
                && amazons_move(
                    &mut self.boardstate,
                    &mut self.src_square,
                    &mut self.dst_square,
                ) == 1
            {
                if amazons_shoot(&mut self.boardstate, &mut self.dst_square, &mut shot) == 1 {
                    swapPlayer(&mut self.boardstate.currentPlayer);
                    return true;
                } else {
                    amazons_move(
                        &mut self.boardstate,
                        &mut self.dst_square,
                        &mut self.src_square,
                    );
                }
            }
        }
        false
    }

    fn game_winner(&mut self) -> SquareState {
        let (mut ws, mut bs) = (0, 0);
        let (winner, moves_left) = unsafe {
            (
                boardstate_winner(&mut self.boardstate, &mut ws, &mut bs),
                playerHasValidMove(&mut self.boardstate, self.boardstate.currentPlayer) == 1,
            )
        };
        if winner != SquareState_EMPTY {
            self.highlight_regions = moves_left;
            self.white_squares = ws as u32;
            self.black_squares = bs as u32;
            if ws == bs {
                return if winner == SquareState_BLACK {
                    SquareState_WHITE
                } else {
                    SquareState_BLACK
                };
            }
            return winner;
        }
        if moves_left {
            return SquareState_EMPTY;
        }
        if self.boardstate.currentPlayer == SquareState_BLACK {
            SquareState_WHITE
        } else {
            SquareState_BLACK
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
            match &self.state {
                ClickableState::GameInProgress => {
                    ui.heading("Game In Progress");
                    if self.boardstate.currentPlayer == SquareState_WHITE {
                        ui.label("Bows to move");
                    } else {
                        ui.label("Spears to move");
                    }
                    if ui.button("Undo last selection").clicked() {
                        if self.clicked_square > 0 {
                            self.clicked_square -= 1;
                        }
                    }
                    if ui.button("Stop Game").clicked() {
                        self.state = ClickableState::Idle;
                    }
                }
                #[allow(non_upper_case_globals)]
                ClickableState::GameOver(winner) => {
                    ui.heading("Game Over!");
                    match *winner {
                        SquareState_WHITE => {
                            ui.label("Bows win!");
                            if self.highlight_regions {
                                ui.label(format!(
                                    "Controlled squares: {} - {}",
                                    self.white_squares, self.black_squares
                                ));
                            }
                        }
                        SquareState_BLACK => {
                            ui.label("Spears win!");
                            if self.highlight_regions {
                                ui.label(format!(
                                    "Controlled squares: {} - {}",
                                    self.black_squares, self.white_squares
                                ));
                            }
                        }
                        _ => (),
                    }
                    if ui.button("OK").clicked() {
                        self.state = ClickableState::Idle;
                    }
                }
                ClickableState::Idle => {
                    ui.heading("Settings");

                    number_setting(ui, &mut self.white_amazons, 0, 10, "Player 1 pieces");
                    number_setting(ui, &mut self.black_amazons, 0, 10, "Player 2 pieces");
                    number_setting(ui, &mut self.board_width, 2, 20, "Board width");
                    number_setting(ui, &mut self.board_height, 2, 20, "Board height");

                    if ui.button("Set player 1 starting positions").clicked() {
                        self.white_starting.clear();
                        self.state = ClickableState::PickingWhite;
                    }

                    if ui.button("Set player 2 starting positions").clicked() {
                        self.black_starting.clear();
                        self.state = ClickableState::PickingBlack;
                    }

                    if ui.button("Revert to default parameters").clicked() {
                        *self = AmazonsGame::default();
                    }

                    if ui.button("New Game").clicked() {
                        self.new_game();
                    }
                }
                ClickableState::PickingWhite | ClickableState::PickingBlack => {
                    ui.heading("Pick Starting Locations");
                    if self.state == ClickableState::PickingWhite {
                        ui.label("Click starting locations for player 1");
                        ui.label(format!(
                            "{}/{} positions chosen",
                            self.white_starting.len(),
                            self.white_amazons
                        ));
                    } else {
                        ui.label("Click starting positions for player 2");
                        ui.label(format!(
                            "{}/{} positions chosen",
                            self.black_starting.len(),
                            self.black_amazons
                        ));
                    }
                    if ui.button("Undo last selection").clicked() {
                        if self.state == ClickableState::PickingWhite {
                            self.white_starting.pop();
                        } else {
                            self.black_starting.pop();
                        }
                    }
                }
                ClickableState::InvalidConfig(reason) => {
                    ui.heading("Invalid Configuration");
                    ui.label(reason);
                    if ui.button("OK").clicked() {
                        self.state = ClickableState::Idle;
                    }
                }
            }

            let sep = Separator::default().spacing(12.).horizontal();
            ui.add(sep);

            if ui.button("Quit").clicked() {
                frame.quit();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Game of the Amazons");
            let (response, painter) =
                ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
                response.rect,
            );
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                if response.clicked() {
                    let canvas_pos = to_screen.inverse() * pointer_pos;
                    let square_size = self.square_size();
                    let x = (canvas_pos.x / square_size).floor() as u32;
                    let y = (canvas_pos.y / square_size).floor() as u32;
                    match self.state {
                        ClickableState::GameInProgress => {
                            let acceptable = match self.clicked_square {
                                0 => self.set_src(x, y),
                                1 => self.set_dst(x, y),
                                _ => {
                                    if self.move_amazon(x, y) {
                                        let winner = self.game_winner();
                                        #[allow(non_upper_case_globals)]
                                        match winner {
                                            SquareState_WHITE | SquareState_BLACK => {
                                                self.state = ClickableState::GameOver(winner)
                                            }
                                            _ => (),
                                        }
                                        true
                                    } else {
                                        false
                                    }
                                }
                            };
                            if acceptable {
                                self.clicked_square = (self.clicked_square + 1) % 3;
                            }
                        }
                        ClickableState::PickingWhite => {
                            if x < self.board_width && y < self.board_height {
                                self.white_starting.push((x, y));
                                if self.white_starting.len() == self.white_amazons as usize {
                                    self.state = ClickableState::Idle;
                                }
                            }
                        }
                        ClickableState::PickingBlack => {
                            if x < self.board_width && y < self.board_height {
                                self.black_starting.push((x, y));
                                if self.black_starting.len() == self.black_amazons as usize {
                                    self.state = ClickableState::Idle;
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
            self.draw_board(&painter, to_screen, frame);
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
