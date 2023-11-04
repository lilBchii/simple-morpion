use ggez::event::{self, EventHandler, MouseButton};
use ggez::graphics::{self, Color};
use ggez::{Context, ContextBuilder, GameResult};
use glam::Vec2;

use std::{env, path};

const CELL_SIZE: f32 = 150.0;
const BG_COLOR: (u8, u8, u8) = (30, 30, 38);

//(top/right/left padding, bottom padding)
const PADDING: (f32, f32) = (50.0, 50.0);

const SCREEN_SIZE: (f32, f32) = (
    (CELL_SIZE * 3.0) + (2.0 * PADDING.0),
    (CELL_SIZE * 3.0) + PADDING.0 + PADDING.1,
);

const DESIRED_FPS: u32 = 18;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Player {
    X,
    O,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellState {
    Occupied(Player),
    Free,
}

#[derive(Debug)]
enum GameState {
    Tie,
    Win(Player),
    Continue,
}

struct Morpion {
    board: [CellState; 9],
    state: GameState,
    last_play: Player,
    meshes: (graphics::Mesh, graphics::Image, graphics::Image),
    clicked: (bool, Option<usize>),
}

impl Morpion {
    pub fn new(ctx: &mut Context) -> GameResult<Morpion> {
        let grid = make_grid_lines(ctx, 6.5, Color::from_rgb(55, 60, 75), PADDING, CELL_SIZE)?;
        let circle = graphics::Image::from_path(ctx, "/circle.png")?;
        let cross = graphics::Image::from_path(ctx, "/cross.png")?;
        Ok(Morpion {
            board: [CellState::Free; 9],
            state: GameState::Continue,
            last_play: Player::X,
            meshes: (grid, cross, circle),
            clicked: (false, None),
        })
    }
    pub fn all_occupied(&self) -> bool {
        let mut b = true;
        for cell in self.board {
            if cell == CellState::Free {
                b = false;
            }
        }
        b
    }
    pub fn is_won(&self) -> bool {
        let player = CellState::Occupied(self.last_play);

        (self.board[0] == player && self.board[1] == player && self.board[2] == player)
            || (self.board[3] == player && self.board[4] == player && self.board[5] == player)
            || (self.board[6] == player && self.board[7] == player && self.board[8] == player)
            || (self.board[0] == player && self.board[3] == player && self.board[6] == player)
            || (self.board[1] == player && self.board[4] == player && self.board[7] == player)
            || (self.board[2] == player && self.board[5] == player && self.board[8] == player)
            || (self.board[0] == player && self.board[4] == player && self.board[8] == player)
            || (self.board[2] == player && self.board[4] == player && self.board[6] == player)
    }
}

impl EventHandler for Morpion {
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, Color::from_rgb(BG_COLOR.0, BG_COLOR.1, BG_COLOR.2));
        // Grid
        canvas.draw(&self.meshes.0, graphics::DrawParam::default());
        // Crosses and Circles
        for (index, cell) in self.board.iter().enumerate() {
            match cell {
                CellState::Free => {}
                CellState::Occupied(Player::X) => {
                    canvas.draw(
                        &self.meshes.1,
                        graphics::DrawParam::new().dest(Vec2::new(
                            PADDING.0 + ((index as u32 % 3) as f32) * CELL_SIZE,
                            PADDING.0 + (((index - index % 3) / 3) as f32) * CELL_SIZE,
                        )),
                    );
                }
                CellState::Occupied(Player::O) => {
                    canvas.draw(
                        &self.meshes.2,
                        graphics::DrawParam::new().dest(Vec2::new(
                            PADDING.0 + ((index % 3) as f32) * CELL_SIZE,
                            PADDING.0 + (((index - index % 3) / 3) as f32) * CELL_SIZE,
                        )),
                    );
                }
            }
        }
        canvas.finish(ctx)
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(DESIRED_FPS) {
            match self.state {
                GameState::Continue => {
                    if self.clicked.0 {
                        let index = self.clicked.1.unwrap();
                        if self.board[index] == CellState::Free {
                            match self.last_play {
                                Player::X => {
                                    self.board[index] = CellState::Occupied(Player::O);
                                    self.last_play = Player::O;
                                }
                                Player::O => {
                                    self.board[index] = CellState::Occupied(Player::X);
                                    self.last_play = Player::X;
                                }
                            }
                        }
                    }
                    if self.is_won() {
                        self.state = GameState::Win(self.last_play);
                    } else if self.all_occupied() {
                        self.state = GameState::Tie;
                    }
                }
                GameState::Tie => {
                    println!("Tie");
                }
                _ => {
                    println!("{:?}", self.state);
                }
            }
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.clicked = (true, Some(id_from_coord(x, y, PADDING, CELL_SIZE)));
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) -> GameResult {
        self.clicked = (false, None);
        Ok(())
    }
}

fn id_from_coord(x: f32, y: f32, anchor: (f32, f32), cellsize: f32) -> usize {
    let h_padd = ((x - anchor.0) / cellsize) as usize + 1;
    let v_padd = ((y - anchor.1) / cellsize) as usize + 1;
    3 * v_padd - (3 - h_padd) - 1
}

// New mesh for the 3x3 grid
fn make_grid_lines(
    ctx: &mut Context,
    width: f32,
    color: Color,
    anchor: (f32, f32),
    cellsize: f32,
) -> GameResult<graphics::Mesh> {
    let l = &mut graphics::MeshBuilder::new();
    l.line(
        &[
            Vec2::new(anchor.0 + cellsize, anchor.1),
            Vec2::new(anchor.0 + cellsize, anchor.1 + cellsize * 3.0),
        ],
        width,
        color,
    )?;
    l.line(
        &[
            Vec2::new(anchor.0 + 2.0 * cellsize, anchor.1),
            Vec2::new(anchor.0 + 2.0 * cellsize, anchor.1 + cellsize * 3.0),
        ],
        width,
        color,
    )?;
    l.line(
        &[
            Vec2::new(anchor.0, anchor.1 + cellsize),
            Vec2::new(anchor.0 + 3.0 * cellsize, anchor.1 + cellsize),
        ],
        width,
        color,
    )?;
    l.line(
        &[
            Vec2::new(anchor.0, anchor.1 + 2.0 * cellsize),
            Vec2::new(anchor.0 + 3.0 * cellsize, anchor.1 + 2.0 * cellsize),
        ],
        width,
        color,
    )?;
    Ok(graphics::Mesh::from_data(ctx, l.build()))
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (mut ctx, events_loop) = ggez::ContextBuilder::new("Morpion", "lilBchii")
        .add_resource_path(resource_dir)
        .window_setup(ggez::conf::WindowSetup::default().title("Morpion"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    let state = Morpion::new(&mut ctx).unwrap();
    event::run(ctx, events_loop, state)
}
