pub mod game_manager;

use std::cell::RefCell;
use std::ops::Mul;
use std::rc::Rc;

use sola_raylib::prelude::*;

pub trait GameObject {
    fn start(&mut self) {}
    fn update(&mut self, delta_time: f32) {}
    fn update_with_input(&mut self, delta_time: f32, rl_input: &RaylibHandle) {
        self.update(delta_time);
    }
    fn render(&self, rl_draw: &mut RaylibDrawHandle);
}

impl<T: GameObject> GameObject for Rc<RefCell<T>> {
    fn start(&mut self) {
        RefCell::borrow_mut(self).start();
    }

    fn update(&mut self, delta_time: f32) {
        RefCell::borrow_mut(self).update(delta_time);
    }

    fn update_with_input(&mut self, delta_time: f32, rl_input: &RaylibHandle) {
        RefCell::borrow_mut(self).update_with_input(delta_time, rl_input);
    }

    fn render(&self, rl_draw: &mut RaylibDrawHandle) {
        RefCell::borrow(self).render(rl_draw);
    }
}
