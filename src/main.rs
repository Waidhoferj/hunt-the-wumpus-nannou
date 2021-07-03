use std::collections::{HashMap, HashSet};
use std::fmt;

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

impl fmt::Display for TileHints {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TileHints::Stench => write!(f, "{}", "Stench"),
            TileHints::Wind => write!(f, "{}", "Wind"),
            TileHints::Glitter => write!(f, "{}", "Glitter"),
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
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
            visited: false,
        }
    }

    fn draw(&self, draw: &Draw, rect: &Rect, textures: &HashMap<TileState, wgpu::Texture>) {
        let tile_rect = draw.rect().xy(rect.xy()).wh(rect.wh());
        if self.visited {
            tile_rect
                .color(rgba(0.3, 0.3, 0.3, 1.0))
                .stroke(rgba(0.2, 0.2, 0.2, 1.0));
        } else {
            tile_rect
                .color(rgba(0.0, 0.0, 0.0, 1.0))
                .stroke(rgba(0.2, 0.2, 0.2, 1.0));
            return;
        }

        match self.state {
            TileState::Gold => {
                draw.ellipse()
                    .xy(rect.xy())
                    .radius(rect.w() / 3.0)
                    .color(YELLOW);
            }
            TileState::Hole => {
                draw.ellipse()
                    .radius(rect.w() / 3.0)
                    .xy(rect.xy())
                    .color(rgb(0.0, 0.0, 0.0));
            }
            TileState::Wumpus => {
                draw.ellipse()
                    .radius(rect.w() / 3.0)
                    .xy(rect.xy())
                    .color(RED);
            }
            TileState::Player => {
                let texture = textures.get(&TileState::Player).unwrap();
                draw.texture(texture).xy(rect.xy()).wh(rect.wh());
            }
            _ => (),
        }

        let hint_text = self.hints.iter().fold(String::from(""), |hints, hint| {
            format!("{}\n{}", hints, hint)
        });

        let text_box = rect.pad(3.0);
        draw.text(&hint_text)
            .color(WHITE)
            .font_size(10)
            .xy(text_box.xy())
            .wh(text_box.wh())
            .align_text_top()
            .left_justify();
    }
}
struct Model {
    board: Vec<Vec<Tile>>,
    player_pos: [usize; 2],
    score: i32,
    total_score: i32,
    textures: HashMap<TileState, wgpu::Texture>,
}

impl Model {
    fn new(board_size: usize) -> Self {
        let mut board = Model::create_board(board_size);

        board[0][0].state = TileState::Player;
        board[0][0].visited = true;

        apply_tile_hints(&mut board);

        let total_score = board.iter().fold(0, |sum, arr| {
            sum + arr.iter().fold(0, |sum, tile| {
                sum + if tile.state == TileState::Gold { 1 } else { 0 }
            })
        });

        Model {
            board,
            player_pos: [0, 0],
            score: 0,
            total_score,
            textures: HashMap::new(),
        }
    }

    fn create_board(board_size: usize) -> Vec<Vec<Tile>> {
        let mut rng = thread_rng();

        (0..board_size)
            .map(|_| {
                (0..board_size)
                    .map(|_| match rng.gen_range(0.0, 1.0) {
                        x if x < 0.7 => TileState::Empty,
                        x if x < 0.9 => TileState::Hole,
                        x if x < 0.95 => TileState::Wumpus,
                        x if x <= 1.0 => TileState::Gold,
                        _ => TileState::Empty,
                    })
                    .map(|state| Tile::new(state))
                    .collect()
            })
            .collect()
    }
}

fn main() {
    nannou::app(model).run();
}

fn event(app: &App, m: &mut Model, e: WindowEvent) {
    let old_pos = m.player_pos.clone();
    match e {
        KeyPressed(key) => match key {
            Key::Up if m.player_pos[1] < m.board.len() - 1 => m.player_pos[1] += 1,
            Key::Down if m.player_pos[1] > 0 => m.player_pos[1] -= 1,
            Key::Left if m.player_pos[0] > 0 => m.player_pos[0] -= 1,
            Key::Right if m.player_pos[0] < m.board.len() - 1 => m.player_pos[0] += 1,
            _ => (),
        },
        _ => (),
    }

    match m.board[m.player_pos[1]][m.player_pos[0]].state {
        TileState::Gold => {
            m.score += 1;
            m.board[m.player_pos[1]][m.player_pos[0]]
                .hints
                .remove(&TileHints::Glitter);
        }
        TileState::Hole | TileState::Wumpus => *m = model(app),
        _ => (),
    }

    if old_pos[0] != m.player_pos[0] || old_pos[1] != m.player_pos[1] {
        m.board[old_pos[1]][old_pos[0]].state = TileState::Empty;
        let cur_tile = &mut m.board[m.player_pos[1]][m.player_pos[0]];
        cur_tile.state = TileState::Player;
        cur_tile.visited = true;
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
            if board[row][col].state == TileState::Gold {
                board[row][col].hints.insert(TileHints::Glitter);
            }
            [(c - 1, r), (c + 1, r), (c, r - 1), (c, r + 1)]
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
    app.new_window().event(event).view(view).build().unwrap();
    let assets = app.assets_path().unwrap();
    let player_texture = wgpu::Texture::from_path(app, assets.join("player.png")).unwrap();
    let mut model = Model::new(10);
    model.textures.insert(TileState::Player, player_texture);
    model
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(WHITE);
    let draw = app.draw();
    let win = app.window_rect();
    let board_dim = model.board.len();
    let board_size = win.w().min(win.h());
    let half_board = board_size / 2.0;
    let tile_size = board_size / board_dim as f32;
    // Draw grid
    for row in 0..board_dim {
        for col in 0..board_dim {
            let y = map_range(row as f32, -1.0, board_dim as f32, -half_board, half_board);
            let x = map_range(col as f32, -1.0, board_dim as f32, -half_board, half_board);
            let rect =
                Rect::from_x_y_w_h(x, y, tile_size, tile_size).pad((tile_size / 10.0).min(10.0));
            let tile = &model.board[row][col];
            tile.draw(&draw, &rect, &model.textures);
        }
    }

    let text_rect = win.pad(10.);
    draw.text(format!("Score: {}/{}", model.score, model.total_score).as_str())
        .font_size(16)
        .align_text_top()
        .left_justify()
        .xy(text_rect.xy())
        .wh(text_rect.wh())
        .color(BLACK);

    draw.to_frame(app, &frame).unwrap();
}
