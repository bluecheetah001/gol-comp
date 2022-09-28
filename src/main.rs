#![feature(int_log)]

use ggez::event;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use glam::*;

use board::{Board, Pos, State};

mod basic;
mod board;
mod macrocell;

struct GlobalState {
    pos_x: f32,
    board: Board,
}

impl GlobalState {
    fn new() -> GameResult<GlobalState> {
        let mut board = Board::new();
        let pos = Pos::new(2, -5);
        let state = State::new(5);
        board.set(pos, state);
        assert_eq!(board.get(pos), state);

        Ok(GlobalState { pos_x: 0.0, board })
    }
}

impl event::EventHandler<ggez::GameError> for GlobalState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            100.0,
            2.0,
            Color::WHITE,
        )?;
        graphics::draw(ctx, &circle, (Vec2::new(self.pos_x, 380.0),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (ctx, event_loop) = cb.build()?;
    let state = GlobalState::new()?;
    event::run(ctx, event_loop, state)
}
