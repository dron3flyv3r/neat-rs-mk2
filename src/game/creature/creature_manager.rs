use std::cell::RefCell;
use std::rc::Rc;

use crate::game::creature::creature::Creature;
use crate::game::world::world::World;
use crate::game_engine::GameObject;

pub struct CreatureManager {
    creatures: Vec<Creature>,
    world: Rc<RefCell<World>>,
}

impl CreatureManager {
    pub fn new(world: Rc<RefCell<World>>) -> Self {
        Self {
            creatures: Vec::new(),
            world,
        }
    }
}

impl GameObject for CreatureManager {
    fn start(&mut self) {
        let creature = Creature::default();

        self.creatures.push(creature);
    }

    fn update_with_input(
        &mut self,
        delta_time: f32,
        rl_input: &sola_raylib::prelude::RaylibHandle,
    ) {
        for creature in &mut self.creatures {
            creature.update(delta_time, rl_input, &mut self.world.borrow_mut());
        }
    }

    fn render(&self, rl_draw: &mut sola_raylib::prelude::RaylibDrawHandle) {
        for creature in &self.creatures {
            creature.render(rl_draw);
        }
    }
}
