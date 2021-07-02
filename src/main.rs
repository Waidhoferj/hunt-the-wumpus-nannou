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
    Player,
}
struct Tile {
    state: TileState,
    hints: HashSet<TileHints>,
    visited: bool,
}

impl Tile {
    fn new(state: TileState) -> Self {
        Tile {
            state,
            hints: HashSet::new(),
            visited: true,
        }
    }

    fn draw(&self, draw: &Draw, rect: &Rect) {
        let tile_rect = draw.rect().xy(rect.xy()).wh(rect.wh());
        if self.visited {
            tile_rect
                .color(rgba(0.3, 0.3, 0.3, 1.0))
                .stroke(rgba(0.2, 0.2, 0.2, 1.0));
        } else {
            tile_rect
                .color(rgba(0.0, 0.0, 0.0, 1.0))
                .stroke(rgba(0.2, 0.2, 0.2, 1.0));
        }

        match self.state {
            TileState::Gold => {
                draw.ellipse()
                    .xy(rect.xy())
                    .radius(rect.w() / 2.0)
                    .color(rgb(0.0, 1.0, 0.0));
            }
            TileState::Hole => {
                draw.ellipse()
                    .radius(rect.w() / 2.0)
                    .xy(rect.xy())
                    .color(rgb(0.0, 0.0, 0.0));
            }
            TileState::Wumpus => {
                draw.ellipse()
                    .radius(rect.w() / 2.0)
                    .xy(rect.xy())
                    .color(rgb(1.0, 0.0, 0.0));
            }
            TileState::Player => {
                draw.ellipse()
                    .radius(rect.w() / 2.0)
                    .xy(rect.xy())
                    .color(rgb(1.0, 0.0, 1.0));
            }
            _ => (),
        }
    }
}
struct Model {
    board: Vec<Vec<Tile>>,
    player_pos: [usize; 2],
    score: i32,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn event(_app: &App, model: &mut Model, e: WindowEvent) {
    let old_pos = model.player_pos.clone();
    match e {
        KeyPressed(key) => match key {
            Key::Up if model.player_pos[1] < model.board.len() - 1 => model.player_pos[1] += 1,
            Key::Down if model.player_pos[1] > 0 => model.player_pos[1] -= 1,
            Key::Left if model.player_pos[0] > 0 => model.player_pos[0] -= 1,
            Key::Right if model.player_pos[0] < model.board.len() - 1 => model.player_pos[0] += 1,
            _ => (),
        },
        MouseMoved(e) => println!("{:?}", e),
        _ => (),
    }

    match model.board[model.player_pos[1]][model.player_pos[0]].state {
        TileState::Gold => {
            model.score += 1;
        }

        TileState::Wumpus => println!("Game over"),
        TileState::Hole => println!("Game over"),
        _ => (),
    }

    if old_pos[0] != model.player_pos[0] || old_pos[1] != model.player_pos[1] {
        model.board[old_pos[1]][old_pos[0]].state = TileState::Empty;
        model.board[model.player_pos[1]][model.player_pos[0]].state = TileState::Player;
    }
}

fn apply_tile_hints(board: &mut Vec<Vec<Tile>>) {
    let board_size = board.len();

    let get_hint = |tile: &TileState| -> Option<TileHints> {
        match tile {
            TileState::Hole => Some(TileHints::Wind),
            TileState::Wumpus => Some(TileHints::Stench),
            _ => None,
        }
    };

    for row in 0..board_size {
        for col in 0..board_size {
            let r: isize = row as isize;
            let c: isize = col as isize;
            let size: isize = board_size as isize;
            [(c - 1, 0), (c + 1, 0), (0, r - 1), (0, r + 1)]
                .iter()
                .for_each(|neighbor| match neighbor {
                    (x, y) if *x >= 0 && *y >= 0 && *x < size && *y < size => {
                        let x = *x as usize;
                        let y = *y as usize;
                        if let Some(hint) = get_hint(&board[y][x].state) {
                            board[row][col].hints.insert(hint);
                        };
                    }
                    _ => (),
                });
        }
    }
}

fn model(app: &App) -> Model {
    let mut rng = thread_rng();
    let board_size = 20;

    let mut board: Vec<Vec<Tile>> = (0..board_size)
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

    board[0][0].state = TileState::Player;

    apply_tile_hints(&mut board);

    app.new_window().event(event).view(view).build().unwrap();

    Model {
        board,
        player_pos: [0, 0],
        score: 0,
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(WHITE);
    let draw = app.draw();
    let win = app.window_rect();
    let board_size = model.board.len();
    let half_board = board_size as f32 / 2.0;
    let tile_size = win.w().min(win.h()) / board_size as f32;
    // Draw grid
    for row in 0..board_size {
        for col in 0..board_size {
            let y = map_range(row, 0, board_size, -half_board, half_board);
            let x = map_range(row, 0, board_size, -half_board, half_board);
            let rect = Rect::from_x_y_w_h(x, y, tile_size, tile_size).pad(2.0);
            let tile = &model.board[row][col];
            tile.draw(&draw, &rect);
        }
    }

    draw.rect().x_y(-1.0, -1.0).w_h(10.0, 10.0).color(RED);
    draw.to_frame(app, &frame).unwrap();
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}
