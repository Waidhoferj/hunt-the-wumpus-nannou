use std::collections::HashSet;
use std::fmt;
use std::iter;

use nannou::{
    prelude::*,
    rand::{thread_rng, Rng},
};

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
enum TileHint {
    Stench,
    Wind,
    Glitter,
}

impl fmt::Display for TileHint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TileHint::Stench => write!(f, "{}", "Stench"),
            TileHint::Wind => write!(f, "{}", "Wind"),
            TileHint::Glitter => write!(f, "{}", "Glitter"),
        }
    }
}
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct GameTextures {
    player_front: wgpu::Texture,
    player_side: wgpu::Texture,
    player_back: wgpu::Texture,
    wumpus: wgpu::Texture,
    chest: wgpu::Texture,
    tile: wgpu::Texture,
}

impl GameTextures {
    fn new(app: &App) -> Self {
        let resources_dir = app.assets_path().unwrap();
        GameTextures {
            player_front: wgpu::Texture::from_path(app, resources_dir.join("player_front.png"))
                .unwrap(),
            player_back: wgpu::Texture::from_path(app, resources_dir.join("player_back.png"))
                .unwrap(),
            player_side: wgpu::Texture::from_path(app, resources_dir.join("player_side.png"))
                .unwrap(),
            wumpus: wgpu::Texture::from_path(app, resources_dir.join("wumpus.png")).unwrap(),
            chest: wgpu::Texture::from_path(app, resources_dir.join("chest.png")).unwrap(),
            tile: wgpu::Texture::from_path(app, resources_dir.join("tile.png")).unwrap(),
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
enum TileState {
    Hole,
    Wumpus,
    Gold,
    Empty,
}

impl TileState {
    fn draw(&self, draw: &Draw, rect: &Rect, textures: &GameTextures) {
        let rect = rect.pad(3.);
        match self {
            TileState::Gold => {
                draw.texture(&textures.chest).xy(rect.xy()).wh(rect.wh());
            }
            TileState::Hole => {
                draw.ellipse()
                    .radius(rect.w() / 3.0)
                    .xy(rect.xy())
                    .color(rgb(0.0, 0.0, 0.0));
            }
            TileState::Wumpus => {
                draw.texture(&textures.wumpus).xy(rect.xy()).wh(rect.wh());
            }
            _ => (),
        }
    }
}

struct Player {
    position: [usize; 2],
    heading: Direction,
    ammo: usize,
}

impl Player {
    fn new() -> Self {
        Player {
            position: [0, 0],
            heading: Direction::Down,
            ammo: 3,
        }
    }

    fn draw(&self, draw: &Draw, rect: &Rect, textures: &GameTextures) {
        let mut dir: f32 = 1.;
        let player_texture = match self.heading {
            Direction::Up => &textures.player_back,
            Direction::Down => &textures.player_front,
            Direction::Right => {
                dir = -1.;
                &textures.player_side
            }
            Direction::Left => &textures.player_side,
        };

        let [w, h] = player_texture.size();
        let aspect = w as f32 / h as f32;
        draw.texture(player_texture)
            .xy(rect.xy())
            .w_h(rect.w() * aspect * dir, rect.h());
    }

    fn shoot(&mut self, board: &mut Board) {
        if self.ammo == 0 {
            return;
        }
        self.ammo -= 1;

        // create next tuple based on direction
        //check if wumpus
        // if so return

        let trajectory: Vec<(usize, usize)> = match self.heading {
            Direction::Down => iter::repeat(self.position[0])
                .zip((0..self.position[1]).rev())
                .collect(),
            Direction::Up => iter::repeat(4 as usize)
                .zip(self.position[1]..board.len())
                .collect(),
            Direction::Left => ((0..self.position[0]).rev())
                .zip(iter::repeat(self.position[1]))
                .collect(),
            Direction::Right => (self.position[0]..board.len())
                .zip(iter::repeat(self.position[1]))
                .collect(),
        };
        let struck_wumpus_pos = trajectory
            .iter()
            .find(|(x, y)| board.get_tile(*x, *y).unwrap().state == TileState::Wumpus);

        if let Some((x, y)) = struck_wumpus_pos {
            board.get_tile_mut(*x, *y).unwrap().state = TileState::Empty;

            board.neighbor_indices(*x, *y).iter().for_each(|(x, y)| {
                board
                    .get_tile_mut(*x, *y)
                    .unwrap()
                    .hints
                    .remove(&TileHint::Stench);
            })
        }
    }
}

struct Board {
    tiles: Vec<Vec<Tile>>,
}

impl Board {
    fn new(board_size: usize) -> Self {
        let mut rng = thread_rng();

        let tiles = (0..board_size)
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
            .collect();

        let mut board = Board { tiles };
        board.update_hints();
        board
    }

    fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        self.tiles.get_mut(y).and_then(|row| row.get_mut(x))
    }

    fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(y).and_then(|row| row.get(x))
    }

    fn get_neighbors(&self, x: usize, y: usize) -> Vec<&Tile> {
        self.neighbor_indices(x, y)
            .iter()
            .filter_map(|(x, y)| self.get_tile(*x, *y))
            .collect()
    }

    fn neighbor_indices(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let r: isize = y as isize;
        let c: isize = x as isize;
        let size = self.tiles.len() as isize;
        [(c - 1, r), (c + 1, r), (c, r - 1), (c, r + 1)]
            .iter()
            .filter_map(move |neighbor| match neighbor {
                (x, y) if *x >= 0 && *y >= 0 && *x < size && *y < size => {
                    let x = *x as usize;
                    let y = *y as usize;
                    Some((x, y))
                }
                _ => None,
            })
            .collect()
    }

    fn update_hints(&mut self) {
        let get_hint = |tile: &TileState| -> Option<TileHint> {
            match tile {
                TileState::Hole => Some(TileHint::Wind),
                TileState::Wumpus => Some(TileHint::Stench),
                _ => None,
            }
        };

        for y in 0..self.len() {
            for x in 0..self.len() {
                let tile = self.get_tile_mut(x, y).unwrap();
                if tile.state == TileState::Gold {
                    tile.hints.insert(TileHint::Glitter);
                }

                let hints: Vec<TileHint> = self
                    .get_neighbors(x, y)
                    .iter()
                    .filter_map(|neighbor| get_hint(&neighbor.state))
                    .collect();
                hints.iter().for_each(|hint| {
                    self.get_tile_mut(x, y).unwrap().hints.insert(*hint);
                })
            }
        }
    }

    fn get_total_score(&self) -> i32 {
        self.tiles.iter().fold(0, |sum, arr| {
            sum + arr.iter().fold(0, |sum, tile| {
                sum + if tile.state == TileState::Gold { 1 } else { 0 }
            })
        })
    }

    fn len(&self) -> usize {
        self.tiles.len()
    }
}
struct Tile {
    state: TileState,
    hints: HashSet<TileHint>,
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

    fn draw(&self, draw: &Draw, rect: &Rect, textures: &GameTextures) {
        if self.visited {
            draw.texture(&textures.tile).xy(rect.xy()).wh(rect.wh());
        } else {
            draw.rect()
                .xy(rect.xy())
                .wh(rect.wh())
                .color(rgba(0.0, 0.0, 0.0, 1.0))
                .stroke(rgba(0.2, 0.2, 0.2, 1.0));
            return;
        }

        self.state.draw(draw, rect, textures);

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
    board: Board,
    player: Player,
    score: usize,
    total_score: usize,
    textures: GameTextures,
}

impl Model {
    fn new(app: &App, board_size: usize) -> Self {
        let board = Board::new(board_size);
        let total_score = board.get_total_score() as usize;

        Model {
            board,
            score: 0,
            total_score,
            player: Player::new(),
            textures: GameTextures::new(&app),
        }
    }
}

fn main() {
    nannou::app(model).run();
}

fn event(app: &App, m: &mut Model, e: WindowEvent) {
    match e {
        KeyPressed(key) => match key {
            Key::Up if m.player.position[1] < m.board.len() - 1 => {
                if m.player.heading == Direction::Up {
                    m.player.position[1] += 1
                } else {
                    m.player.heading = Direction::Up
                }
            }
            Key::Down if m.player.position[1] > 0 => {
                if m.player.heading == Direction::Down {
                    m.player.position[1] -= 1
                } else {
                    m.player.heading = Direction::Down;
                }
            }
            Key::Left if m.player.position[0] > 0 => {
                if m.player.heading == Direction::Left {
                    m.player.position[0] -= 1
                } else {
                    m.player.heading = Direction::Left;
                }
            }
            Key::Right if m.player.position[0] < m.board.len() - 1 => {
                if m.player.heading == Direction::Right {
                    m.player.position[0] += 1
                } else {
                    m.player.heading = Direction::Right;
                }
            }
            Key::Space
                if m.board
                    .get_tile(m.player.position[0], m.player.position[1])
                    .unwrap()
                    .state
                    == TileState::Gold =>
            {
                m.score += 1;
                let cur_tile = m
                    .board
                    .get_tile_mut(m.player.position[0], m.player.position[1])
                    .unwrap();
                cur_tile.hints.remove(&TileHint::Glitter);
                cur_tile.state = TileState::Empty;
            }

            Key::Return => {
                m.player.shoot(&mut m.board);
            }

            _ => (),
        },
        _ => (),
    }

    match m
        .board
        .get_tile(m.player.position[0], m.player.position[1])
        .unwrap()
        .state
    {
        TileState::Hole | TileState::Wumpus => *m = model(app),
        _ => (),
    }

    m.board
        .get_tile_mut(m.player.position[0], m.player.position[1])
        .unwrap()
        .visited = true;
}

fn model(app: &App) -> Model {
    app.new_window().event(event).view(view).build().unwrap();
    Model::new(app, 15)
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
            let rect = Rect::from_x_y_w_h(x, y, tile_size, tile_size);
            let tile = model.board.get_tile(col, row).unwrap();
            tile.draw(&draw, &rect, &model.textures);
        }
    }
    // Draw player
    let y = map_range(
        model.player.position[1] as f32,
        -1.0,
        board_dim as f32,
        -half_board,
        half_board,
    );
    let x = map_range(
        model.player.position[0] as f32,
        -1.0,
        board_dim as f32,
        -half_board,
        half_board,
    );
    let rect = Rect::from_x_y_w_h(x, y, tile_size, tile_size);
    model.player.draw(&draw, &rect, &model.textures);

    //Draw score
    let text_rect = win.pad(10.);
    draw.text(
        format!(
            "Score: {}/{} \n Ammo: {}",
            model.score, model.total_score, model.player.ammo
        )
        .as_str(),
    )
    .font_size(16)
    .align_text_top()
    .left_justify()
    .xy(text_rect.xy())
    .wh(text_rect.wh())
    .color(BLACK);

    draw.to_frame(app, &frame).unwrap();
}
