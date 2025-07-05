use std::f32::consts::PI;

use godot::global::Key;
use godot::prelude::*;
use godot::classes::*;

use crate::utils::*;

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
pub struct Thruster {
    #[export]
    thrust: f32,
    base: Base<RigidBody3D>,
}

#[godot_api]
impl IRigidBody3D for Thruster {
    fn init(base: Base<RigidBody3D>) -> Self {
        godot_print!("Player initialized."); // Prints to the Godot console

        Self {
            thrust: 0.,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let delta32 = delta as f32;

        let forward = euler_to_dir_godot(self.base().get_global_rotation());

        let thrust_vec = self.thrust * forward;
        godot_print!("thrust_vec {} {}: {}, {}, {}", self.base().get_name(), self.thrust, thrust_vec.x, thrust_vec.y, thrust_vec.z);
        
        self.base_mut().apply_force(thrust_vec);
    }
}