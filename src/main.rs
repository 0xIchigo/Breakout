use macroquad::prelude::*;

const PADDLE_SIZE: Vec2 = const_vec2!([150f32, 40f32]);
const PADDLE_SPEED: f32 = 700f32;
const BLOCK_SIZE: Vec2 = const_vec2!([100f32, 40f32]);
const BALL_SIZE: f32 = 50f32;
const BALL_SPEED: f32 = 450f32;

pub enum GameState {
    Menu,
    Game,
    Won,
    Dead,
}

struct Paddle {
    rect: Rect,
}

impl Paddle {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                screen_width() * 0.5f32 - PADDLE_SIZE.x*0.5f32,
                screen_height() - 100f32,
                PADDLE_SIZE.x,
                PADDLE_SIZE.y,
            ),
        }
    }

    pub fn update(&mut self, dt: f32) {
        let x_move = match (is_key_down(KeyCode::Left), is_key_down(KeyCode::Right)) {
            (true, false) => -1f32,
            (false, true) => 1f32,
            _ => 0f32,
        };

        self.rect.x += x_move * dt * PADDLE_SPEED;

        // If we hit the left wall
        if self.rect.x < 0f32 {
            self.rect.x = 0f32;
        }

        // If we hit the right wall
        if self.rect.x > screen_width() - self.rect.w {
            self.rect.x = screen_width() - self.rect.w;
        }
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, BLACK);
    }
}

#[derive(PartialEq)]
pub enum BlockType {
    Regular,
    SpawnBallOnDeath,
}

struct Block {
    rect: Rect,
    lives: i32,
    block_type: BlockType,
}

impl Block {
    pub fn new(pos: Vec2, block_type: BlockType) -> Self {
        Self {
            rect: Rect::new(pos.x, pos.y, BLOCK_SIZE.x, BLOCK_SIZE.y),
            lives: 2,
            block_type,
        }
    }

    pub fn draw(&self) {
        let color = match self.block_type {
            BlockType::Regular => match self.lives {
                2 => RED,
                _ => ORANGE,
            },
            BlockType::SpawnBallOnDeath => GREEN,
        };

        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, color);
    }
}

struct Ball {
    rect: Rect,
    vel: Vec2,
}

impl Ball {
    pub fn new(pos: Vec2) -> Self {
        Self {
            rect: Rect::new(pos.x, pos.y, BALL_SIZE, BALL_SIZE),
            // Randomizing vec values can mess up the length of the vector
            // We call normalize to ensure the length is always one
            vel: vec2(rand::gen_range(-1f32, 1f32), 1f32).normalize(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.rect.x += self.vel.x * dt * BALL_SPEED;
        self.rect.y += self.vel.y * dt * BALL_SPEED;

        // If we hit the left wall
        if self.rect.x < 0f32 {
            self.vel.x = 1f32;
        }

        // If we hit the right wall
        if self.rect.x > screen_width() - self.rect.w {
            self.vel.x = -1f32;
        }

        // If we hit the ceiling
        if self.rect.y < 0f32 {
            self.vel.y = 1f32;
        }
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, WHITE);
    }
}

// AABB (axis-aligned bounding box) collision with positional correction
// Essentially, AABB is a rectangular collision shape aligned to the base axes of the scene
// which aligns to the x and y axis
fn resolve_collision(a: &mut Rect, vel: &mut Vec2, b: &Rect) -> bool {
    // intersection returns an Option of the value that represents the area created by two overlapping rects
    let intersection = match a.intersect(*b) { // Dereference as intersection takes an owned value of a Rect
        Some(intersection) => intersection,
        None => return false, //Early exit
    };
    
    let a_center = a.point() + a.size() * 0.5f32;
    let b_center = b.point() + b.size() * 0.5f32;
    let to = b_center - a_center;

    // The signum function is a mathematical function that extracts the sign of any real number 
    // This helps with collission direction as we can determine its horizontal direction
    let to_signum = to.signum();

    match intersection.w > intersection.h {
        true => {
            //Bounce on the y axis
            a.y -= to_signum.y * intersection.h;
            vel.y = -to_signum.y * vel.y.abs();
        },
        false => {
             // Bounce on the x axis
            a.x -= to_signum.x * intersection.w;
            vel.x = -to_signum.x * vel.x.abs();
        }
    }
    true
}

// Resets the game after a player loses and wishes to play again
fn reset_game(
    score: &mut i32,
    player_lives: &mut i32,
    blocks: &mut Vec<Block>,
    balls: &mut Vec<Ball>,
    paddle: &mut Paddle,
) {
    *paddle = Paddle::new();
    *score = 0;
    *player_lives = 3;
    balls.clear();
    balls.push(Ball::new(vec2(screen_width() * 0.5f32 - BALL_SIZE * 0.5f32, screen_height() * 0.5f32,)));
    blocks.clear();
    init_blocks(blocks);
}

// Creates the board
fn init_blocks(blocks: &mut Vec<Block>) {
    let (width, height) = (6, 5);
    let padding = 5f32;
    let total_block_size = BLOCK_SIZE + vec2(padding, padding);
    let board_start_pos = vec2((screen_width() - (total_block_size.x * width as f32)) * 0.5f32, 50f32);

    for i in 0..width * height {
        let block_x = (i % width) as f32 * total_block_size.x;
        let block_y = (i / width) as f32 * total_block_size.y;

        blocks.push(Block::new(board_start_pos + vec2(block_x, block_y), BlockType::Regular));
    }

    for _ in 0..3 {
        let rand_index = rand::gen_range(0, blocks.len());
        blocks[rand_index].block_type = BlockType::SpawnBallOnDeath;
    }
}

fn draw_title_text(text: &str, font: Font) {
    let dims = measure_text(text, Some(font), 50u16, 1.0f32);
    draw_text_ex(
        text,
        screen_width() * 0.5f32 - dims.width * 0.5f32,
        screen_height() * 0.5f32 - dims.height * 0.5f32,
        TextParams {
            font,
            font_size: 50u16,
            color: WHITE,
            ..Default::default()
        },
    );
}

#[macroquad::main("Breakout")]
async fn main() {
    let font = load_ttf_font("res/OpenSans-Regular.ttf").await.unwrap();
    let mut game_state = GameState::Menu;
    let mut score = 0;
    let mut player_lives = 3;

    let mut paddle = Paddle::new();
    let mut blocks = Vec::new();
    let mut balls = Vec::new();

    balls.push(Ball::new(vec2(screen_width() * 0.5f32, screen_height() * 0.6f32,)));
    init_blocks(&mut blocks);

    loop {
        match game_state {
            GameState::Menu => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Game;
                }
            },
            GameState::Game => {
                paddle.update(get_frame_time());

                for ball in balls.iter_mut() {
                    ball.update(get_frame_time());
                }

                let mut spawn_later = vec![];
                for ball in balls.iter_mut() {
                    resolve_collision(&mut ball.rect, &mut ball.vel, &paddle.rect);
                    for block in blocks.iter_mut() {
                        // Checks if the ball collided with the paddle
                        if resolve_collision(&mut ball.rect, &mut ball.vel, &block.rect) {
                            block.lives -= 1;
                            if block.lives <= 0 {
                                score += 10;

                                // Spawns a new ball if it is of the special block type
                                if block.block_type == BlockType::SpawnBallOnDeath {
                                    spawn_later.push(Ball::new(ball.rect.point()));
                                }
                            }
                        }
                    }
                }
                for ball in spawn_later.into_iter() {
                    balls.push(ball);
                }

                let balls_len = balls.len();
                // Remove balls that went past the paddle
                balls.retain(|ball| ball.rect.y < screen_height());

                //If the last ball went past the paddle the player loses a life
                let removed_balls = balls_len - balls.len();
                if removed_balls > 0 && balls.is_empty() {
                    player_lives -= 1;
                    balls.push(Ball::new(
                        paddle.rect.point()
                            + vec2(paddle.rect.w * 0.5f32 - BALL_SIZE * 0.5f32, -50f32),
                    ));

                    if player_lives <= 0 {
                        game_state = GameState::Dead;
                    }
                }
                // Remove blocks that were destroyed - if lambda is true then it stays, if false it is removed from the vector
                blocks.retain(|block| block.lives > 0);

                if blocks.is_empty() {
                    game_state = GameState::Won;
                }
            },
            GameState::Won | GameState::Dead => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Menu;
                    reset_game(
                        &mut score,
                        &mut player_lives,
                        &mut blocks,
                        &mut balls,
                        &mut paddle,
                    );
                }
            }
        }

        clear_background(DARKGRAY);
        paddle.draw();

        for block in blocks.iter() {
            block.draw();
        }

        for ball in balls.iter() {
            ball.draw();
        }

        match game_state {
            GameState::Menu => {
                draw_title_text("Press SPACE to start", font);
            },
            GameState::Game => {
                let score_text = format!("Score: {}", score);
                let score_text_dim = measure_text(&score_text, Some(font), 30u16, 1.0);

                // Displays the score at the top of the screen
                draw_text_ex(
                    &score_text,
                    screen_width() * 0.5f32 - score_text_dim.width * 0.5f32,
                    40.0,
                    TextParams {
                        font,
                        font_size: 30u16,
                        color: WHITE,
                        ..Default::default()
                    },
                );

                // Displays the player's remaining lives at the top of the screen
                draw_text_ex(
                    &format!("Lives: {}", player_lives),
                    30.0,
                    40.0,
                    TextParams {
                        font,
                        font_size: 30u16,
                        color: WHITE,
                        ..Default::default()
                    },
                );
            },
            GameState::Won => {
                draw_title_text(&format!("You won with a score of {}! ", score), font);
            },
            GameState::Dead => {
                draw_title_text(&format!("You lost with a score of {}!", score), font);
            }
        }

        next_frame().await;
    }
}
