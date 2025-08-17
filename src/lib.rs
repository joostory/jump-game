mod utils;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, KeyboardEvent};

#[derive(PartialEq)]
enum GameState {
    Playing,
    GameOver,
}

struct Player {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    velocity_x: f64,
    velocity_y: f64,
    is_jumping: bool,
    is_moving_left: bool,
    is_moving_right: bool,
}

impl Player {
    fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            width: 50.0,
            height: 50.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            is_jumping: false,
            is_moving_left: false,
            is_moving_right: false,
        }
    }

    fn draw(&self, context: &CanvasRenderingContext2d, camera_y: f64) {
        context.set_fill_style_str("green");
        context.fill_rect(self.x, self.y - camera_y, self.width, self.height);
    }
}

struct Block {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Block {
    fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    fn draw(&self, context: &CanvasRenderingContext2d, camera_y: f64) {
        context.set_fill_style_str("brown");
        context.fill_rect(self.x, self.y - camera_y, self.width, self.height);
    }
}

struct Game {
    state: GameState,
    player: Player,
    blocks: Vec<Block>,
    context: CanvasRenderingContext2d,
    width: u32,
    height: u32,
    camera_y: f64,
    score: i32,
}

impl Game {
    fn new(context: CanvasRenderingContext2d, width: u32, height: u32) -> Self {
        let mut blocks = Vec::new();
        // Create the ground block
        blocks.push(Block::new(0.0, (height - 20) as f64, width as f64, 20.0));

        // Create random blocks
        for i in 1..10 {
            let y = (height - 120 * i) as f64;
            let x = js_sys::Math::random() * (width - 100) as f64;
            blocks.push(Block::new(x, y, 100.0, 20.0));
        }

        Self {
            state: GameState::Playing,
            player: Player::new((width / 2) as f64 - 25.0, (height - 70) as f64),
            blocks,
            context,
            width,
            height,
            camera_y: 0.0,
            score: 0,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn update(&mut self) {
        if self.state != GameState::Playing {
            return;
        }

        let gravity = 0.5;
        let move_speed = 5.0;

        if self.player.is_moving_left {
            self.player.velocity_x = -move_speed;
        } else if self.player.is_moving_right {
            self.player.velocity_x = move_speed;
        } else {
            self.player.velocity_x = 0.0;
        }

        self.player.velocity_y += gravity;
        self.player.x += self.player.velocity_x;
        self.player.y += self.player.velocity_y;

        // Wall collision
        if self.player.x < 0.0 {
            self.player.x = 0.0;
        }
        if self.player.x + self.player.width > self.width as f64 {
            self.player.x = self.width as f64 - self.player.width;
        }

        // Block collision
        for block in &self.blocks {
            let player_bottom = self.player.y + self.player.height;
            if self.player.velocity_y > 0.0 &&
               self.player.x < block.x + block.width &&
               self.player.x + self.player.width > block.x &&
               player_bottom >= block.y &&
               player_bottom <= block.y + self.player.velocity_y {
                self.player.y = block.y - self.player.height;
                self.player.velocity_y = 0.0;
                self.player.is_jumping = false;
            }
        }

        // Camera follow
        if self.player.y - self.camera_y < self.height as f64 / 2.0 {
            self.camera_y = self.player.y - self.height as f64 / 2.0;
        }

        // Game Over condition
        if self.player.y - self.camera_y > self.height as f64 {
            self.state = GameState::GameOver;
        }

        // Update score
        let new_score = (-(self.player.y - (self.height - 70) as f64) / 10.0) as i32;
        if new_score > self.score {
            self.score = new_score;
        }

        // Generate new blocks
        if self.blocks.last().unwrap().y - self.camera_y > -100.0 {
            let last_block = self.blocks.last().unwrap();
            let last_x = last_block.x;
            let y = last_block.y - 120.0;

            // Ensure the next block is reachable and not at the screen edges
            let max_jump_dist = 200.0; // Max horizontal distance player can jump
            let relative_min_x = last_x - max_jump_dist;
            let relative_max_x = last_x + last_block.width + max_jump_dist;

            let screen_margin = 50.0;
            let screen_min_x = screen_margin;
            let screen_max_x = self.width as f64 - 100.0 - screen_margin; // 100 is block width

            let min_x = relative_min_x.max(screen_min_x);
            let max_x = relative_max_x.min(screen_max_x);

            let x = if min_x < max_x {
                min_x + js_sys::Math::random() * (max_x - min_x)
            } else {
                // Fallback to center if the range is invalid
                (self.width / 2 - 50) as f64
            };

            self.blocks.push(Block::new(x, y, 100.0, 20.0));
        }

        // Remove old blocks
        let camera_y = self.camera_y;
        let height = self.height as f64;
        self.blocks.retain(|block| block.y - camera_y < height);
    }

    fn draw(&self) {
        self.context.set_fill_style_str("#87CEEB"); // Sky blue background
        self.context.fill_rect(0.0, 0.0, self.width as f64, self.height as f64);

        for block in &self.blocks {
            block.draw(&self.context, self.camera_y);
        }

        self.player.draw(&self.context, self.camera_y);

        self.context.set_fill_style_str("black");
        self.context.set_font("24px Arial");
        self.context.set_text_align("start"); // Reset text alignment
        self.context.fill_text(&format!("Score: {}", self.score), 10.0, 30.0).unwrap();

        if self.state == GameState::GameOver {
            self.context.set_fill_style_str("rgba(0, 0, 0, 0.5)");
            self.context.fill_rect(0.0, 0.0, self.width as f64, self.height as f64);

            self.context.set_fill_style_str("white");
            self.context.set_font("60px Arial");
            self.context.set_text_align("center");
            self.context.fill_text("Game Over", self.width as f64 / 2.0, self.height as f64 / 2.0 - 40.0).unwrap();

            self.context.set_font("30px Arial");
            self.context.fill_text(&format!("Final Score: {}", self.score), self.width as f64 / 2.0, self.height as f64 / 2.0 + 20.0).unwrap();
            self.context.fill_text("Click to Restart", self.width as f64 / 2.0, self.height as f64 / 2.0 + 70.0).unwrap();
        }
    }

    fn start_move(&mut self, direction: &str) {
        if self.state != GameState::Playing {
            return;
        }
        match direction {
            "left" => self.player.is_moving_left = true,
            "right" => self.player.is_moving_right = true,
            _ => {}
        }
    }

    fn stop_move(&mut self, direction: &str) {
        match direction {
            "left" => self.player.is_moving_left = false,
            "right" => self.player.is_moving_right = false,
            _ => {}
        }
    }

    fn jump(&mut self) {
        if self.state == GameState::Playing && !self.player.is_jumping {
            self.player.velocity_y = -15.0;
            self.player.is_jumping = true;
        }
    }
}

// Use a global mutable state for the game
thread_local! {
    static GAME: Rc<RefCell<Option<Game>>> = Rc::new(RefCell::new(None));
}

#[wasm_bindgen]
pub fn resize(width: u32, height: u32) {
    GAME.with(|game_rc| {
        if let Some(game) = &mut *game_rc.borrow_mut() {
            game.resize(width, height);
        }
    });
}

#[wasm_bindgen]
pub fn start_move(direction: String) {
    GAME.with(|game_rc| {
        if let Some(game) = &mut *game_rc.borrow_mut() {
            game.start_move(&direction);
        }
    });
}

#[wasm_bindgen]
pub fn stop_move(direction: String) {
    GAME.with(|game_rc| {
        if let Some(game) = &mut *game_rc.borrow_mut() {
            game.stop_move(&direction);
        }
    });
}

#[wasm_bindgen]
pub fn jump() {
    GAME.with(|game_rc| {
        if let Some(game) = &mut *game_rc.borrow_mut() {
            game.jump();
        }
    });
}

#[wasm_bindgen]
pub fn handle_key_down(event: KeyboardEvent) {
    GAME.with(|game_rc| {
        if let Some(game) = &mut *game_rc.borrow_mut() {
            match event.key().as_str() {
                "ArrowLeft" => game.start_move("left"),
                "ArrowRight" => game.start_move("right"),
                " " | "ArrowUp" => game.jump(),
                _ => {}
            }
        }
    });
}

#[wasm_bindgen]
pub fn handle_key_up(event: KeyboardEvent) {
    GAME.with(|game_rc| {
        if let Some(game) = &mut *game_rc.borrow_mut() {
            match event.key().as_str() {
                "ArrowLeft" => game.stop_move("left"),
                "ArrowRight" => game.stop_move("right"),
                _ => {}
            }
        }
    });
}

#[wasm_bindgen]
pub fn handle_click() {
    GAME.with(|game_rc| {
        if let Some(game) = &mut *game_rc.borrow_mut() {
            if game.state == GameState::GameOver {
                // Reset the game by creating a new instance
                let new_game = Game::new(game.context.clone(), game.width, game.height);
                *game = new_game;
            }
        }
    });
}

#[wasm_bindgen]
pub fn start_game(width: u32, height: u32) -> Result<(), JsValue> {
    utils::set_panic_hook();

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let canvas = document
        .get_element_by_id("game-canvas")
        .ok_or_else(|| JsValue::from_str("Canvas not found"))?
        .dyn_into::<HtmlCanvasElement>()?;

    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Context not supported"))?
        .dyn_into::<CanvasRenderingContext2d>()?;

    // Initialize the game state with initial canvas size
    GAME.with(|game_rc| {
        *game_rc.borrow_mut() = Some(Game::new(context, width, height));
    });

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let game_loop_window = window.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        GAME.with(|game_rc| {
            if let Some(game) = &mut *game_rc.borrow_mut() {
                game.update();
                game.draw();
            }
        });

        game_loop_window.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }) as Box<dyn FnMut()>));

    let initial_f = g.borrow();
    window.request_animation_frame(initial_f.as_ref().unwrap().as_ref().unchecked_ref())?;

    Ok(())
}