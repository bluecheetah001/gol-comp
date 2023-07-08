#![feature(local_key_cell_methods)]
#![feature(const_option)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
// lints
#![warn(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::box_default,
    clippy::similar_names
)]

mod image;

use std::fs;
use std::num::NonZeroU64;
use std::time::{Duration, Instant};

use eframe::egui;
use eframe::epaint::Color32;
use image::with_image;
use node::{Node, Population, Pos, Quadrant, Rect};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt()
        .with_target(true)
        // .with_timer(LocalTime)
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 1000.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

struct MyApp {
    board: Board,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            board: Board::new_centered(
                Node::read_from_bytes(&fs::read("test.mc").unwrap()).unwrap(),
                Node::read_from_bytes(&fs::read("glider.mc").unwrap()).unwrap(),
            ),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut self.board);
        });
    }
}

enum CursorMode {
    Toggle,
    Select(Option<Pos>, Rect),
    Paste,
}

struct Board {
    node: Node,

    generation: u64,
    step_size: NonZeroU64,
    /// >= 0 is step_size * 2^play_power generations per frame
    /// <= -1 is step_size generations every 2^(play_power+4) seconds
    play_power: i8,
    last_time: Instant,
    play: bool,

    center: Pos,
    center_fine: egui::Vec2,
    /// pixels_per_cell is 2^zoom
    zoom_power: i8,
    // /// linear scale on top of pixels_per_cell to avoid zooming too quickly
    // /// is between .707 and 1.414
    // zoom_fine: f32,
    cursor: CursorMode,
    clipboard: Node,
    // TODO move clipboard back to here
}
impl Board {
    const MIN_ZOOM: i8 = -70;
    const MAX_ZOOM: i8 = 5;
    const MIN_PLAY: i8 = -5;
    const MIN_PLAY_NANOS: u64 = 1_000_000_000;
    const MAX_PLAY: i8 = 16;
    const FOOTER_HEIGHT: f32 = 10.0;

    pub fn new_centered(node: Node, clipboard: Node) -> Self {
        Self {
            node,

            generation: 0,
            step_size: NonZeroU64::new(1).unwrap(),
            play_power: 0,
            last_time: Instant::now(),
            play: true,

            // TODO infer default center based on node bounding box
            center: Pos { x: 0, y: 0 },
            center_fine: egui::vec2(0.0, 0.0),
            // TODO infer default zoom based on ui rect and node bounding box
            zoom_power: 2,

            cursor: CursorMode::Paste,
            clipboard,
        }
    }

    fn slow_play_delay(&self) -> Duration {
        Duration::from_nanos(Board::MIN_PLAY_NANOS / (1 << (self.play_power - Board::MIN_PLAY)))
    }
}
impl egui::Widget for &mut Board {
    #[allow(clippy::too_many_lines)] // TODO not sure how to refactor yet
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // create painter and leave space for footer
        let (response, painter) = ui.allocate_painter(
            ui.available_size() - egui::vec2(0.0, Board::FOOTER_HEIGHT),
            egui::Sense::click_and_drag(),
        );

        // handle zoom inputs
        // if response.has_focus() { // TODO not sure how to manage focus
        ui.input(|input| {
            // handle zoom inputs
            if input.key_pressed(egui::Key::I) && self.zoom_power < Board::MAX_ZOOM {
                self.zoom_power += 1;
            }
            if input.key_pressed(egui::Key::O) && self.zoom_power > Board::MIN_ZOOM {
                self.zoom_power -= 1;
            }
            // TODO `input.zoom_delta()` for touch screens
            // TODO `input.scroll_delta` for touch screens since i can't check for scroll wheel specifically to turn into zoom
        });
        // }

        // handle speed inputs
        let mut step_once = false;
        ui.input(|input| {
            if input.key_pressed(egui::Key::Enter) {
                self.play = !self.play;
            }
            if input.key_pressed(egui::Key::Space) {
                step_once = !self.play;
                self.play = false;
            }
            if input.key_pressed(egui::Key::PlusEquals) && self.play_power < Board::MAX_PLAY {
                self.play_power += 1;
            }
            if input.key_pressed(egui::Key::Minus) && self.play_power > Board::MIN_PLAY {
                self.play_power -= 1;
            }
        });

        // handle screen drag inputs
        let pixels_per_cell = 2.0_f32.powi(self.zoom_power.into());
        let pixels_per_point = painter.ctx().pixels_per_point();
        let points_per_cell = pixels_per_cell / pixels_per_point;

        #[allow(clippy::cast_possible_truncation)] // floats should be small
        // TODO also use ui.input.scroll_delta
        if response.dragged_by(egui::PointerButton::Secondary) {
            self.center_fine -= response.drag_delta() * pixels_per_point / pixels_per_cell;

            let center_x_floor = self.center_fine.x.floor();
            self.center_fine.x -= center_x_floor;
            self.center.x += center_x_floor as i64;

            let center_y_floor = self.center_fine.y.floor();
            self.center_fine.y -= center_y_floor;
            self.center.y += center_y_floor as i64;
        }

        let center_point = painter
            .round_pos_to_pixels(painter.clip_rect().center() - self.center_fine * points_per_cell);

        // handle cursor mode inputs
        ui.input(|input| {
            if input.key_pressed(egui::Key::C) || input.key_pressed(egui::Key::X) {
                if let CursorMode::Select(_, rect) = self.cursor {
                    if !rect.is_empty() {
                        let node = self.node.clip(rect);
                        // TODO using offset_norm to center isn't really ideal
                        // but is better than nothing
                        let (_, node) = node.offset_norm();
                        self.clipboard = node;
                    }
                }
            }
            if input.key_pressed(egui::Key::D) || input.key_pressed(egui::Key::X) {
                if let CursorMode::Select(_, rect) = self.cursor {
                    if !rect.is_empty() {
                        self.node = self.node.clear(rect);
                    }
                }
            }
            if input.key_pressed(egui::Key::V) {
                self.cursor = CursorMode::Paste;
            }
            if input.key_pressed(egui::Key::R) {
                if input.modifiers.shift {
                    self.clipboard = self.clipboard.rotate_ccw();
                } else {
                    self.clipboard = self.clipboard.rotate_cw();
                }
            }
            if input.key_pressed(egui::Key::F) {
                if input.modifiers.shift {
                    self.clipboard = self.clipboard.flip_v();
                } else {
                    self.clipboard = self.clipboard.flip_h();
                }
            }
            if input.key_pressed(egui::Key::E) {
                self.cursor = CursorMode::Toggle;
            }
            if input.key_pressed(egui::Key::V) {
                self.cursor = CursorMode::Paste;
            }
            if input.key_pressed(egui::Key::Escape) {
                self.cursor = CursorMode::Select(None, Rect::NOTHING);
            }
        });

        // handle clicks
        #[allow(clippy::cast_possible_truncation)] // did floor, floats should be small
        let hover = if self.zoom_power >= 0 {
            response.hover_pos().map(|mouse| {
                let mouse_cell = ((mouse - center_point) / points_per_cell).floor();
                self.center
                    + Pos {
                        x: mouse_cell.x as i64,
                        y: mouse_cell.y as i64,
                    }
            })
        } else {
            // TODO large area hover mode?
            None
        };
        if let Some(hover) = hover {
            match &mut self.cursor {
                CursorMode::Toggle => {
                    if response.clicked_by(egui::PointerButton::Primary) {
                        // TODO toggle fn?
                        self.node = self.node.set(hover, !self.node.get(hover));
                    }
                }
                CursorMode::Paste => {
                    if response.clicked_by(egui::PointerButton::Primary) {
                        // TODO xor?
                        self.node = self.node.or(&self.clipboard.offset(hover));
                    }
                }
                CursorMode::Select(pos, rect) => {
                    if response.drag_started_by(egui::PointerButton::Primary) {
                        *pos = Some(hover);
                    }
                    if let Some(pos) = pos {
                        if response.dragged_by(egui::PointerButton::Primary) {
                            *rect = Rect::new(*pos, hover);
                        }
                    }
                }
            }
        }

        // handle update
        let now = Instant::now();
        let steps = if step_once {
            self.step_size.get()
        } else if !self.play {
            0
        } else if self.play_power >= 0 {
            self.step_size.get() * (1 << self.play_power)
        } else if now >= self.last_time + self.slow_play_delay() {
            self.step_size.get()
        } else {
            0
        };
        if let Some(steps) = NonZeroU64::new(steps) {
            self.last_time = now;
            self.generation += steps.get();
            self.node = self.node.step_non_zero(steps);
            if self.play {
                if self.play_power >= 0 {
                    ui.ctx().request_repaint();
                } else {
                    ui.ctx().request_repaint_after(self.slow_play_delay());
                }
            }
        } else if self.play && self.play_power < 0 {
            let remaining = self.last_time + self.slow_play_delay() - now;
            ui.ctx().request_repaint_after(remaining);
        }

        // draw node
        if self.zoom_power >= 0 {
            paint_node(
                &painter,
                center_point,
                points_per_cell,
                -self.center,
                &self.node,
                Color32::WHITE,
            );
            if let Some(hover) = hover {
                match &self.cursor {
                    CursorMode::Toggle => {}
                    CursorMode::Paste => {
                        paint_node(
                            &painter,
                            center_point,
                            points_per_cell,
                            -self.center + hover,
                            &self.clipboard,
                            Color32::from_rgba_premultiplied(0, 0, 255, 128),
                        );
                    }
                    CursorMode::Select(_, rect) => {
                        paint_rect(
                            &painter,
                            center_point,
                            points_per_cell,
                            *rect,
                            Color32::from_rgba_premultiplied(0, 128, 255, 128),
                        );
                    }
                }
            }
        } else {
            let node = self.node.reduce_by(self.zoom_power.unsigned_abs());

            paint_node(
                &painter,
                center_point,
                1.0 / pixels_per_point,
                -self.center,
                &node,
                Color32::WHITE,
            );
        }

        // draw footer
        ui.horizontal_centered(|ui| {
            // TODO backround

            // TODO fixed size so stuff doesn't move around
            let generation = self.generation;
            ui.label(format!("generation: {generation}"));

            if self.play {
                ui.label("running");
            } else {
                ui.label("paused");
            }

            let step_size = self.step_size;
            let play_power = self.play_power;
            if play_power >= 0 {
                ui.label(format!("speed: {step_size}*2^{play_power}/frame"));
            } else {
                let delay_sec = self.slow_play_delay().as_secs_f32();
                ui.label(format!("speed: {step_size}/{delay_sec}s"));
            }

            if let Some(Pos { x, y }) = hover {
                ui.label(format!("mouse: {x},{y}"));
            }
        });

        // TODO this is just the response of the main area, but it should contain the footer?
        response
    }
}
#[allow(clippy::cast_precision_loss)] // positions should be small
fn paint_node(
    painter: &egui::Painter,
    center_point: egui::Pos2,
    points_per_cell: f32,
    pos: Pos,
    node: &Node,
    color: Color32,
) {
    if node.is_empty() {
        return;
    }
    let half_width = node.half_width();
    let min = center_point
        + egui::vec2(
            (pos.x - half_width) as f32 * points_per_cell,
            (pos.y - half_width) as f32 * points_per_cell,
        );
    let max = center_point
        + egui::vec2(
            (pos.x + half_width) as f32 * points_per_cell,
            (pos.y + half_width) as f32 * points_per_cell,
        );
    let rect = egui::Rect::from_min_max(min, max);
    if !painter.clip_rect().intersects(rect) {
        return;
    }
    // TODO move to const with comment saying why 32x32 pixel images
    if node.depth() > 1 {
        let inner = node.inner().unwrap();
        let quarter_width = half_width / 2;
        for q in Quadrant::iter_all() {
            paint_node(
                painter,
                center_point,
                points_per_cell,
                pos + Pos::in_dir(q, quarter_width),
                &inner[q],
                color,
            );
        }
    } else {
        with_image(painter.ctx(), node, |image| {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            painter.image(image, rect, uv, color);
        });
    }
}

#[allow(clippy::cast_precision_loss)] // positions should be small
fn paint_rect(
    painter: &egui::Painter,
    center_point: egui::Pos2,
    points_per_cell: f32,
    rect: Rect,
    color: Color32,
) {
    if rect.is_empty() {
        return;
    }
    let min = center_point
        + egui::vec2(
            rect.west() as f32 * points_per_cell,
            rect.north() as f32 * points_per_cell,
        );
    let max = center_point
        + egui::vec2(
            rect.east() as f32 * points_per_cell,
            rect.south() as f32 * points_per_cell,
        );
    let egui_rect = egui::Rect::from_min_max(min, max);
    painter.rect_filled(egui_rect, 0.0, color);
}
