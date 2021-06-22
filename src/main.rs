use nannou::{
    prelude::*,
    rand::{thread_rng, Rng},
};

enum TileHints {
    Stench,
    Wind,
    Glitter,
}
enum TileState {
    Hole(Vec<TileHints>),
    Wumpus(Vec<TileHints>),
    Gold(Vec<TileHints>),
    Empty(Vec<TileHints>),
}
struct Model {
    board: Vec<Vec<TileState>>,
    player_pos: (i32, i32),
    score: i32,
}

fn main() {
    nannou::app(model)
        .view(view)
        .update(update)
        .event(event)
        .run();
}

fn event(_app: &App, _model: &mut Model, event: WindowEvent) {}

fn model(app: &App, model: &mut Model, update: Update) -> Model {
    let mut rng = thread_rng();
    let board_size = 20;

    let mut board = (0..board_size)
        .map(|_| {
            (0..board_size).map(|_| match rng.gen_range(0.0, 1.0) {
                x if x < 0.5 => TileState::Empty(vec![]),
                x if x < 0.9 => TileState::Hole(vec![]),
                x if x <= 1.0 => TileState::Gold(vec![TileHints::Glitter]),
                _ => TileState::Empty(vec![]),
            })
        })
        .collect();

    State {}
}

fn view() {}

fn update() {}
