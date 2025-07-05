use std::f32::consts::PI;

use godot::global::Key;
use godot::prelude::*;
use godot::classes::*;

use crate::utils::*;

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
pub struct Balloon {
    #[export]
    pub bouyancy: Vector3,
    pub base: Base<RigidBody3D>,
}

#[godot_api]
impl IRigidBody3D for Balloon {
    fn init(base: Base<RigidBody3D>) -> Self {
        godot_print!("Balloon initialized."); // Prints to the Godot console
        
        Self {
            bouyancy: Vector3::new(0., 19., 0.),
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let bouyancy = self.bouyancy.clone();
        self.base_mut().apply_force(bouyancy);
    }
}