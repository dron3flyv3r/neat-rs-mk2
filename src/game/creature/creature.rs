use sola_raylib::prelude::*;

use crate::game::world::world::{Tile, World};
use crate::game_engine::collision::{
    Collision, CollisionCircle, CollisionPolygon, CollisionResult, CollisionShape,
};

pub struct Creature {
    position: Vector2,
    color: Color,
    speed: f32,
    size: f32,
    collision_shape: CollisionShape,
    base_stats: Stats,
    current_stats: Stats,
    consumption_rates: ConsumtionRates,
}

impl Default for Creature {
    fn default() -> Self {
        let position = Vector2 { x: 0.0, y: 0.0 };
        let size = 10.0;

        let collision_shape = CollisionShape::Circle(CollisionCircle::new(position, size));
        Self {
            position,
            size,
            color: Color::BLUE,
            speed: 100.0,
            collision_shape,
            base_stats: Stats {
                health: 100.0,
                hunger: 100.0,
                thirst: 100.0,
            },
            current_stats: Stats {
                health: 100.0,
                hunger: 100.0,
                thirst: 100.0,
            },
            consumption_rates: ConsumtionRates {
                food: 0.5,
                water: 0.5,
                starvation: 0.5,
                dehydration: 1.5,
            },
        }
    }
}

#[derive(Clone)]
pub struct Stats {
    health: f32,
    hunger: f32,
    thirst: f32,
}

pub struct ConsumtionRates {
    food: f32,
    water: f32,
    starvation: f32,
    dehydration: f32,
}

impl Creature {
    pub fn new(
        position: Vector2,
        speed: f32,
        size: f32,
        base_stats: Stats,
        consumption_rates: ConsumtionRates,
    ) -> Self {
        let colors = [Color::PINK, Color::PURPLE];
        let color = *colors.get(rand::random_range(0..colors.len() - 1)).unwrap();

        let collision_shape = CollisionShape::Circle(CollisionCircle::new(position, size));

        Self {
            position,
            current_stats: base_stats.clone(),
            collision_shape,
            base_stats,
            consumption_rates,
            color,
            speed,
            size,
        }
    }

    pub fn update(&mut self, delta_time: f32, rl_input: &RaylibHandle, world: &mut World) {
        // Update creature logic here
        let mut movement_dir = Vector2 { x: 0.0, y: 0.0 };

        if rl_input.is_key_down(KeyboardKey::KEY_W) {
            movement_dir.y -= 1.0;
        }

        if rl_input.is_key_down(KeyboardKey::KEY_S) {
            movement_dir.y += 1.0;
        }

        if rl_input.is_key_down(KeyboardKey::KEY_D) {
            movement_dir.x += 1.0;
        }

        if rl_input.is_key_down(KeyboardKey::KEY_A) {
            movement_dir.x -= 1.0;
        }

        if rl_input.is_key_pressed(KeyboardKey::KEY_E) {
            self.eat(world.consume_food_at(&self.position, &10.0));
            self.drink(world.consume_water_at(&self.position, &10.0));
        }

        movement_dir = movement_dir.normalized();

        let speed = match world.get_tile_at(&self.position).get_or_insert_default() {
            Tile::Sand => self.speed * 0.5, // Slower in water
            _ => self.speed,
        };

        let movement_delta = Vector2 {
            x: movement_dir.x * speed * delta_time,
            y: movement_dir.y * speed * delta_time,
        };

        // Check for collisions with the world
        self.move_and_collide(world, &movement_delta);

        self.current_stats.hunger -= self.consumption_rates.food * delta_time;
        self.current_stats.thirst -= self.consumption_rates.water * delta_time;

        // Ensure stats don't go below zero and apply health penalties if they do
        if self.current_stats.hunger < 0.0 {
            self.current_stats.hunger = 0.0;
            self.current_stats.health -= self.consumption_rates.starvation * delta_time; // Lose health if hunger is zero
        }
        if self.current_stats.thirst < 0.0 {
            self.current_stats.thirst = 0.0;
            self.current_stats.health -= self.consumption_rates.dehydration * delta_time; // Lose health if thirst is zero
        }

        if !self.is_alive() {
            println!("Creature has died.");
        } else {
            // println!(
            //     "Creature Stats - Health: {}, Hunger: {}, Thirst: {}",
            //     self.current_stats.health, self.current_stats.hunger, self.current_stats.thirst
            // );
        }
    }

    pub fn render(&self, rl_draw: &mut RaylibDrawHandle) {
        // Render the creature on the screen
        rl_draw.draw_circle(
            self.position.x as i32,
            self.position.y as i32,
            self.size,
            self.color,
        );
    }

    fn is_alive(&self) -> bool {
        self.current_stats.health > 0.0
    }

    fn eat(&mut self, amount: f32) {
        if self.current_stats.hunger + amount > self.base_stats.hunger {
            self.current_stats.hunger = self.base_stats.hunger;
        } else {
            self.current_stats.hunger += amount;
        }
    }

    fn drink(&mut self, amount: f32) {
        if self.current_stats.thirst + amount > self.base_stats.thirst {
            self.current_stats.thirst = self.base_stats.thirst;
        } else {
            self.current_stats.thirst += amount;
        }
    }

    fn move_and_collide(&mut self, world: &World, delta: &Vector2) {
        // X
        let mut test = self.position;
        test.x += delta.x;

        for tile in &mut world.get_tile_colliders(&self.collision_shape()) {
            if let Some(c) = self.check_collision_at_position(tile, &test) {
                test.x -= c.correction_vector.x;
            }
        }
        self.position.x = test.x;

        // Y
        let mut test = self.position;
        test.y += delta.y;

        for tile in &mut world.get_tile_colliders(&self.collision_shape()) {
            if let Some(c) = self.check_collision_at_position(tile, &test) {
                test.y -= c.correction_vector.y;
            }
        }
        self.position.y = test.y;
    }
}

impl Collision for Creature {
    fn collision_shape(&mut self) -> CollisionShape {
        let collision_shape = match &mut self.collision_shape {
            CollisionShape::Circle(circle) => circle,
            // CollisionShape::Circle(circle) => circle,
            _ => panic!("Creature collision shape must be a rectangle"),
        };
        collision_shape.update_position(self.position);
        collision_shape.update_radius(self.size);

        self.collision_shape.clone()
    }

    fn collision_enabled(&self) -> bool {
        true
    }
}
