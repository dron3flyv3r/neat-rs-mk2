#[macro_use]
mod macros;
mod game;
mod game_engine;
mod trainer;

use std::cell::RefCell;
use std::rc::Rc;

use game::creature::creature_manager::CreatureManager;
use game_engine::game_manager::GameManager;

use sola_raylib::prelude::*;

pub fn run() {
    let world = Rc::new(RefCell::new(game::world::world::World::new(
        (64, 64),
        Vector2 { x: 15.0, y: 15.0 },
    )));
    let creature_manager = CreatureManager::new(Rc::clone(&world));

    let mut gm = game_manager![
        960 x 960,
        game_objects: [
            world,
            creature_manager
        ]
    ];

    gm.run();
}
