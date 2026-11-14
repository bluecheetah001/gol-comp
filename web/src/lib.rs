#![feature(float_conversions)]

use std::num::NonZeroU64;

use node::{Block, Node, Population, Pos, Quadrant, Rect};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Viewer {
    original_base: Node,
    original_actual: Node,
    period: Option<NonZeroU64>,
    base: Node,
    actual: Node,
    step: u64,

    canvas_size: Pos,
    // zoom: i8,
    center: Pos,
}
#[wasm_bindgen]
impl Viewer {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default, reason = "is only for wasm")]
    pub fn new() -> Self {
        let empty = Node::empty(0);
        Self {
            original_base: empty.clone(),
            original_actual: empty.clone(),
            period: None,
            base: empty.clone(),
            actual: empty,
            step: 0,
            // arbitrary values that are expected to be overwritten
            canvas_size: Pos::new(100, 100),
            center: Pos::new(0, 0),
        }
    }
    pub fn load_base(&mut self, source: &str) -> Result<(), String> {
        self.original_base = Node::read_from_string(source).map_err(|e| format!("{e}"))?;
        self.base = self.original_base.step(self.steps_in_period());
        Ok(())
    }
    pub fn load_actual(&mut self, source: &str) -> Result<(), String> {
        self.original_actual = Node::read_from_string(source).map_err(|e| format!("{e}"))?;
        self.actual = self.original_actual.step(self.steps_in_period());
        Ok(())
    }
    pub fn set_period(&mut self, period: Option<f64>) -> Result<(), String> {
        let period = period.map(check_positive_u64).transpose()?;
        let prev_steps_in_period = self.steps_in_period();
        self.period = period;
        self.update_nodes(prev_steps_in_period);
        Ok(())
    }
    pub fn set_step(&mut self, step: f64) -> Result<(), String> {
        let step = check_u64(step)?;
        let prev_steps_in_period = self.steps_in_period();
        self.step = step;
        self.update_nodes(prev_steps_in_period);
        Ok(())
    }
    fn steps_in_period(&self) -> u64 {
        if let Some(period) = self.period {
            self.step % period
        } else {
            self.step
        }
    }
    fn update_nodes(&mut self, prev_steps_in_period: u64) {
        let curr_steps_in_period = self.steps_in_period();
        if curr_steps_in_period >= prev_steps_in_period {
            let new_steps = curr_steps_in_period - prev_steps_in_period;
            self.base = self.base.step(new_steps);
            self.actual = self.actual.step(new_steps);
        } else {
            self.base = self.original_base.step(curr_steps_in_period);
            self.actual = self.original_actual.step(curr_steps_in_period);
        }
    }

    pub fn set_canvas_size(&mut self, canvas_width: f64, canvas_height: f64) -> Result<(), String> {
        self.canvas_size = Pos::new(
            check_positive_i64(canvas_width)?,
            check_positive_i64(canvas_height)?,
        );
        Ok(())
    }
    pub fn set_center(&mut self, x: f64, y: f64) -> Result<(), String> {
        self.center = Pos::new(check_i64(x)?, check_i64(y)?);
        Ok(())
    }
    pub fn get_rgba(&self) -> Vec<u8> {
        let depth = self.base.depth().max(self.actual.depth());
        let base = self.base.center_at_depth(depth);
        let actual = self.actual.center_at_depth(depth);

        // for now assuming pixels are cells
        let min = self.center - (self.canvas_size / 2);
        let max = min + self.canvas_size - Pos::new(1, 1);
        let view = Rect::min_max(min, max);
        ImageBuilder::new(view).fill(&base, &actual).into_rgba()
    }
}

fn check_u64(val: f64) -> Result<u64, String> {
    if val < 0.0 || val.is_nan() {
        return Err("Must be non-negative".to_owned());
    }
    let int_val: u64 = val.to_int_saturating();
    if int_val as f64 != val {
        // technically this is looser, but error is good enough
        return Err("Must be a safe integer".to_owned());
    }
    Ok(int_val)
}
fn check_positive_u64(val: f64) -> Result<NonZeroU64, String> {
    if val <= 0.0 || val.is_nan() {
        return Err("Must be positive".to_owned());
    }
    let int_val: u64 = val.to_int_saturating();
    if int_val as f64 != val {
        // technically this is looser, but error is good enough
        return Err("Must be a safe integer".to_owned());
    }
    Ok(NonZeroU64::try_from(int_val).unwrap())
}
fn check_positive_i64(val: f64) -> Result<i64, String> {
    if val <= 0.0 || val.is_nan() {
        return Err("Must be positive".to_owned());
    }
    let int_val: i64 = val.to_int_saturating();
    if int_val as f64 != val {
        // technically this is looser, but error is good enough
        return Err("Must be a safe integer".to_owned());
    }
    Ok(int_val)
}
fn check_i64(val: f64) -> Result<i64, String> {
    let int_val: i64 = val.to_int_saturating();
    if int_val as f64 != val {
        // technically this is looser, but error is good enough
        return Err("Must be a safe integer".to_owned());
    }
    Ok(int_val)
}

struct ImageBuilder {
    bits: Vec<u8>,
    view: Rect,
}
impl ImageBuilder {
    fn new(view: Rect) -> Self {
        let width: usize = view.width().try_into().unwrap();
        let height: usize = view.height().try_into().unwrap();
        Self {
            bits: vec![0; width * height],
            view,
        }
    }
    fn fill(mut self, base: &Node, actual: &Node) -> Self {
        self.fill_at(base, actual, Pos::new(0, 0));
        self
    }
    fn into_rgba(self) -> Vec<u8> {
        self.bits
            .into_iter()
            .flat_map(|b| match b {
                0 => [226, 232, 240, 255], // dead => white
                1 => [251, 146, 60, 255],  // base only => orange
                2 => [56, 189, 248, 255],  // actual only => blue
                3 => [15, 23, 42, 255],    // alive => black
                _ => unreachable!(),
            })
            .collect()
    }
    fn fill_at(&mut self, base: &Node, actual: &Node, pos: Pos) {
        if base.is_empty() && actual.is_empty() {
            return;
        }
        if !base
            .trivial_bounding_rect()
            .intersects(self.view.offset(pos))
        {
            return;
        }
        let half_width = base.half_width();
        match (base.depth_quad(), actual.depth_quad()) {
            (node::DepthQuad::Inner(_, base), node::DepthQuad::Inner(_, actual)) => {
                for q in Quadrant::iter_all() {
                    self.fill_at(&base[q], &actual[q], pos + Pos::in_dir(q, half_width));
                }
            }
            (node::DepthQuad::Leaf(base), node::DepthQuad::Leaf(actual)) => {
                for q in Quadrant::iter_all() {
                    self.fill_block(base[q], actual[q], pos + Pos::in_dir(q, half_width));
                }
            }
            _ => panic!("inconsistent depth"),
        }
    }
    fn fill_block(&mut self, base: Block, actual: Block, pos: Pos) {
        if !Block::TRIVIAL_BOUNDING_RECT.intersects(self.view.offset(pos)) {
            return;
        }
        self.fill_pixels(base, 1, pos);
        self.fill_pixels(actual.and_not(base), 2, pos);
        self.fill_pixels(base.and_not(actual), 4, pos);
    }
    fn fill_pixels(&mut self, block: Block, mask: u8, pos: Pos) {
        for local_pos in block.iter_alive() {
            let cell_pos = pos + local_pos;
            if self.view.contains(cell_pos) {
                let pixel_pos = cell_pos - self.view.nw();
                let row = pixel_pos.y as usize;
                let col = pixel_pos.x as usize;
                let width = self.view.width() as usize;
                let index = row * width + col;
                self.bits[index] |= mask;
            }
        }
    }
}
