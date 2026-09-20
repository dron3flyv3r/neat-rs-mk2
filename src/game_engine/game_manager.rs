use crate::game_engine::GameObject;
use sola_raylib::prelude::*;

pub struct GameManager {
    game_objects: Vec<Box<dyn GameObject + 'static>>, // [world, creates......]
    rl: RaylibHandle,
    thread: RaylibThread,
    window_size: (i32, i32),
}

impl GameManager {
    pub fn new(window_size: (i32, i32)) -> Self {
        let (rl, thread) = sola_raylib::init()
            .size(window_size.0, window_size.1)
            .title("Neat Game")
            .build();

        Self {
            game_objects: Vec::new(),
            rl,
            thread,
            window_size,
        }
    }

    pub fn run(&mut self) {
        for go in &mut self.game_objects {
            go.start();
        }

        while !self.rl.window_should_close() {
            let delta_time = self.rl.get_frame_time();
            // Update all game objects
            for go in &mut self.game_objects {
                go.update_with_input(delta_time, &self.rl);
            }

            let mut draw_handle = self.rl.begin_drawing(&self.thread);
            // Draw all game objects
            draw_handle.clear_background(Color::LAWNGREEN);
            for go in &mut self.game_objects {
                go.render(&mut draw_handle);
            }
        }
    }

    pub fn add_game_object(&mut self, game_object: Box<dyn GameObject + 'static>) {
        self.game_objects.push(game_object);
    }
}
