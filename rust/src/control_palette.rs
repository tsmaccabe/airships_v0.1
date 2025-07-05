use std::f32::consts::PI;

use godot::global::Key;
use godot::prelude::*;
use godot::classes::*;

use crate::utils::*;

pub struct ControlPalette {
    pub keys: Vec<Key>,
    pub pressed: Vec<bool>,
}

impl ControlPalette {
    pub fn new(keys: Vec<Key>) -> Self {
        ensure_unique_keys(&keys);

        let mut pressed = Vec::new();
        for _ in keys.iter() {
            pressed.push(false);
        }
        Self {keys, pressed}
    }

    pub fn get_value(&mut self, key: Key) -> bool {
        self.pressed[self.keys.iter().position(|r| *r == key).unwrap()]
    }

    pub fn get_values(&mut self, keys: Vec<Key>) -> Vec<bool> {
        ensure_unique_keys(&keys);

        let mut values = Vec::new();

        for key in keys {
            values.push(self.get_value(key));
        };
        values
    }

    pub fn set_value(&mut self, key: Key, pressed: bool) {
        if !self.keys.contains(&key) {
            panic!("ControlPalette: input key not found in this palette.")
        }
        self.pressed[self.keys.iter().position(|r| *r == key).unwrap()] = pressed;
    }

    pub fn set_values(&mut self, keys: Vec<Key>, pressed: Vec<bool>) {
        ensure_unique_keys(&keys);

        for (i, key) in keys.clone().into_iter().enumerate() {
            if !keys.contains(&key) {
                panic!("ControlPalette: input key not found in this palette.")
            }

            self.pressed[i] = pressed[i];
        };
    }
}