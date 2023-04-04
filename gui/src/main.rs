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
mod reduce;

use std::fs;

use eframe::egui;
use image::with_image;
use node::{Node, Population, Pos, Quadrant};
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
            ),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.board.node = self.board.node.step(1);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut self.board);
        });
        ctx.request_repaint();
    }
}

struct Board {
    node: Node,
    center: Pos,
    center_fine: egui::Vec2,
    points_per_cell: f32,
}
impl Board {
    pub fn new_centered(node: Node) -> Self {
        Self {
            node,
            center: Pos { x: 0, y: 0 },
            center_fine: egui::vec2(0.0, 0.0),
            points_per_cell: 4.0,
        }
    }
}
impl egui::Widget for &mut Board {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

        let pixels_per_point = painter.ctx().pixels_per_point();
        let points_per_cell = (self.points_per_cell * pixels_per_point)
            .round()
            .max(1.0) // zooming out isn't supported yet
            / pixels_per_point;

        #[allow(clippy::cast_possible_truncation)] // floats should be small
        if response.dragged_by(egui::PointerButton::Secondary) {
            self.center_fine -= response.drag_delta() / points_per_cell;

            let center_x_floor = self.center_fine.x.floor();
            self.center_fine.x -= center_x_floor;
            self.center.x += center_x_floor as i64;

            let center_y_floor = self.center_fine.y.floor();
            self.center_fine.y -= center_y_floor;
            self.center.y += center_y_floor as i64;
        }

        let center_point = painter
            .round_pos_to_pixels(painter.clip_rect().center() - self.center_fine * points_per_cell);

        // TODO adjust self.points_per_cell and self.center_fine based on above rounding?

        paint_node(
            &painter,
            center_point,
            points_per_cell,
            -self.center,
            &self.node,
        );

        // TODO or handle response internally since this maybe shouldn't be a widget
        // as this needs to do 'complicated' coordinate transforms
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
            );
        }
    } else {
        with_image(painter.ctx(), node, |image| {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            painter.image(image, rect, uv, painter.ctx().style().visuals.text_color());
        });
    }
}
