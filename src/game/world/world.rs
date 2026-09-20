use std::{
    ops::Add,
    time::{Duration, SystemTime},
};

use noise::{NoiseFn, Perlin};
use sola_raylib::prelude::*;

use crate::game_engine::GameObject;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Tile {
    #[default]
    Empty,
    Grass,
    Water,
    Food(f32),
    Sand,
}

pub struct World {
    map: Vec<Vec<Tile>>,
    tile_size: Vector2,
    next_allowed_food: std::time::SystemTime,
}

impl World {
    pub fn new(map_size: (usize, usize), tile_size: Vector2) -> Self {
        Self {
            map: vec![vec![Tile::Empty; map_size.0]; map_size.1],
            tile_size,
            next_allowed_food: std::time::SystemTime::now().add(Duration::from_millis(500)),
        }
    }

    fn generate_map(&mut self) {
        let water_noise = Perlin::new(rand::random::<u32>());

        let water_scale = 15.0;

        for y in 0..self.map.len() {
            for x in 0..self.map[0].len() {
                let water_value = water_noise.get([x as f64 / water_scale, y as f64 / water_scale]);

                if water_value > 0.5 {
                    self.map[y][x] = Tile::Water;
                } else if water_value > 0.2 {
                    self.map[y][x] = Tile::Sand;
                } else {
                    self.map[y][x] = Tile::Grass;
                }
            }
        }
    }

    fn spawn_food(&mut self) {
        for _ in 0..20 {
            let x = rand::random_range(0..self.map.len());
            let y = rand::random_range(0..self.map.len());

            let cell = &mut self.map[y][x];

            if *cell == Tile::Grass {
                *cell = Tile::Food(100.0);
                return;
            }
        }
    }

    fn pixel_to_world(&self, x: f32, y: f32) -> (usize, usize) {
        (
            (x / self.tile_size.x) as usize,
            (y / self.tile_size.y) as usize,
        )
    }

    pub fn get_naboring_tiles(&self, position: Vector2, area: usize) -> Vec<Tile> {
        let (x, y) = self.pixel_to_world(position.x, position.y);
        let mut tiles = Vec::new();

        for dy in -(area as isize)..=(area as isize) {
            for dx in -(area as isize)..=(area as isize) {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0
                    && ny >= 0
                    && (nx as usize) < self.map[0].len()
                    && (ny as usize) < self.map.len()
                {
                    tiles.push(self.map[ny as usize][nx as usize]);
                }
            }
        }

        tiles
    }

    pub fn get_tile_at(&self, position: &Vector2) -> Option<Tile> {
        if position.x < 0.0 || position.y < 0.0 {
            return None;
        }

        let (x, y) = self.pixel_to_world(position.x, position.y);

        if x < self.map[0].len() && y < self.map.len() {
            Some(self.map[y][x])
        } else {
            None
        }
    }

    pub fn consume_food_at(&mut self, position: &Vector2, amount: &f32) -> f32 {
        let (x, y) = self.pixel_to_world(position.x, position.y);

        if x < self.map[0].len()
            && y < self.map.len()
            && let Tile::Food(food_amount) = &mut self.map[y][x]
        {
            let eat = food_amount.min(*amount);

            *food_amount -= eat;

            if *food_amount <= 0.0 {
                self.map[y][x] = Tile::Grass;
            }
            return eat;
        }
        0.0
    }

    pub fn consume_water_at(&mut self, position: &Vector2, amount: &f32) -> f32 {
        let neighbor_tiles = self.get_naboring_tiles(*position, 1);
        let is_water_nearby = neighbor_tiles.contains(&Tile::Water);
        if is_water_nearby {
            return *amount;
        }
        0.0
    }

    pub fn is_tile_walkable(&self, position: &Vector2) -> bool {
        self.get_tile_at(position)
            .is_some_and(|tile| tile != Tile::Water && tile != Tile::Empty)
    }
}

impl GameObject for World {
    fn start(&mut self) {
        self.generate_map();
    }

    fn update(&mut self, delta_time: f32) {
        // Spawn food if nessesary
        let food_amount = self
            .map
            .iter()
            .flatten()
            .filter(|c| matches!(c, Tile::Food(_)))
            .count();

        if food_amount < 20 && SystemTime::now() > self.next_allowed_food {
            self.spawn_food();
            self.next_allowed_food = SystemTime::now().add(Duration::from_millis(500));
        }
    }

    fn render(&self, rl_draw: &mut RaylibDrawHandle) {
        for y in 0..self.map.len() {
            for x in 0..self.map[0].len() {
                let cell = &self.map[y][x];
                let start_pos = Vector2 {
                    x: x as f32 * self.tile_size.x,
                    y: y as f32 * self.tile_size.y,
                };

                let color = match cell {
                    Tile::Empty => Color::DARKGRAY,
                    Tile::Grass => Color::LAWNGREEN,
                    Tile::Food(food_amont) => Color::TOMATO,
                    Tile::Water => Color::DEEPSKYBLUE,
                    Tile::Sand => Color::WHEAT,
                };

                if let Tile::Food(food_amount) = *cell {
                    let radius =
                        food_amount / 100.0 * (self.tile_size.x.min(self.tile_size.y) / 2.0);
                    let center_x = start_pos.x + self.tile_size.x / 2.0;
                    let center_y = start_pos.y + self.tile_size.y / 2.0;
                    rl_draw.draw_circle(
                        center_x as i32,
                        center_y as i32,
                        radius.clamp(2.0, self.tile_size.x.min(self.tile_size.y)),
                        color,
                    );
                } else {
                    rl_draw.draw_rectangle(
                        start_pos.x as i32,
                        start_pos.y as i32,
                        self.tile_size.x as i32,
                        self.tile_size.y as i32,
                        color,
                    );
                }
            }
        }

        // Draw grid lines
        let line_color = Color::new(140, 140, 140, 100);
        for y in 0..self.map.len() {
            let start_pos = Vector2 {
                x: 0.0,
                y: y as f32 * self.tile_size.y,
            };
            let end_pos = Vector2 {
                x: self.map.len() as f32 * self.tile_size.x,
                y: y as f32 * self.tile_size.y,
            };

            rl_draw.draw_line_ex(start_pos, end_pos, 1.0, line_color);
        }

        for x in 0..self.map[0].len() {
            let start_pos = Vector2 {
                x: x as f32 * self.tile_size.x,
                y: 0.0,
            };
            let end_pos = Vector2 {
                x: x as f32 * self.tile_size.x,
                y: self.map.len() as f32 * self.tile_size.y,
            };

            rl_draw.draw_line_ex(start_pos, end_pos, 1.0, line_color);
        }
    }
}
