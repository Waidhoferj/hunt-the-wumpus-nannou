use std::collections::HashSet;

use nannou::{
    prelude::*,
    rand::{thread_rng, Rng},
};

#[derive(Hash, Eq, PartialEq, Debug)]
enum TileHints {
    Stench,
    Wind,
    Glitter,
}
enum TileState {
    Hole,
    Wumpus,
    Gold,
    Empty,
}
struct Tile {
    state: TileState,
    hints: HashSet<TileHints>,
}

impl Tile {
    fn new(state: TileState) -> Self {
        Tile {
            state,
            hints: HashSet::new(),
            visited: bool,
        }
    }
}
struct Model {
    board: Vec<Vec<Tile>>,
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

fn apply_tile_hints(board: &mut Vec<Vec<Tile>>) {
    let board_size = board.len() as isize;

    let set_hints = |x: isize, y: isize| {
        ([(-1, 0), (1, 0), (0, -1), (0, 1)] as [(isize, isize); 4])
            .iter()
            .filter_map(|(dx, dy)| {
                let y = y + dy;
                let x = x + dx;
                if x >= 0 && x < board_size && y >= 0 && y < board_size {
                    Some((x as usize, y as usize))
                } else {
                    None
                }
            })
            .for_each(|(x, y)| match board[y][x] {
                Tile {
                    state: TileState::Gold,
                    hints,
                } => (),
                Tile {
                    state: TileState::Hole,
                    hints,
                } => {
                    hints.insert(TileHints::Wind);
                }
                Tile {
                    state: TileState::Wumpus,
                    hints,
                } => {
                    hints.insert(TileHints::Stench);
                }
                Tile {
                    state: TileState::Empty,
                    hints,
                } => (),
            });
    };

    for row in 0..board_size {
        for col in 0..board_size {
            set_hints(col, row);
        }
    }
}

fn model(app: &App, model: &mut Model, update: Update) -> Model {
    let mut rng = thread_rng();
    let board_size = 20;

    let mut board = (0..board_size)
        .map(|_| {
            (0..board_size)
                .map(|_| match rng.gen_range(0.0, 1.0) {
                    x if x < 0.5 => TileState::Empty,
                    x if x < 0.9 => TileState::Hole,
                    x if x <= 1.0 => TileState::Gold,
                    _ => TileState::Empty,
                })
                .map(|state| Tile::new(state))
                .collect()
        })
        .collect();

    Model {
        board,
        player_pos: (0, 0),
        score: 0,
    }
}


fn draw_unvisited_tile(draw: &Draw, x: usize, y: usize) {
    
}

fn view(app: &App, model: &Model, frame: Frame) {
    let grid_size = [10, 10];
    let grid_gap = [2, 2];
    let draw = app.draw();
    const VISITED_COLOR: Rgba  = rgba(0.3,0.3,0.3, 1.0);
    const HIDDEN_COLOR: Rgba  = rgba(0.0, 0.0, 0.0, 1.0);
    let draw_unvisited_tile = |x: usize, y:usize| {
        draw.rect()
                .x_y(
                    (x * (grid_size[0] + grid_gap[0])) as f32,
                    (y * (grid_size[1] + grid_gap[1])) as f32,
                )
                .w_h(grid_size[0] as f32, grid_size[1] as f32)
                .color(VISITED_COLOR)
                .stroke(rgba(0.2, 0.2, 0.2, 1.0));
    }

    let draw_visited_tile = |x: usize, y:usize| {
        draw.rect()
                .x_y(
                    (x * (grid_size[0] + grid_gap[0])) as f32,
                    (y * (grid_size[1] + grid_gap[1])) as f32,
                )
                .w_h(grid_size[0] as f32, grid_size[1] as f32)
                .color(VISITED_COLOR)
                .stroke(rgba(0.2, 0.2, 0.2, 1.0));
        
    }

    let draw_player = ||
    // Draw grid
    for row in 0..model.board.len() {
        for col in 0..model.board.len() {
            draw.rect()
                .x_y(
                    (col * (grid_size[0] + grid_gap[0])) as f32,
                    (row * (grid_size[1] + grid_gap[1])) as f32,
                )
                .w_h(grid_size[0] as f32, grid_size[1] as f32)
                .rgba(0.5, 1.0, 1.0, 1.0)
                .stroke(BLACK);
        }
    }

    draw.to_frame(app, &frame);
}

fn update() {}
