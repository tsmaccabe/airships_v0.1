use std::f32::consts::PI;

use godot::global::Key;
use godot::prelude::*;
use godot::classes::*;

struct AirshipsGodot;

/* Utilities */
mod utils;
use utils::*;

/* Rust Classes */


/* Godot Classes */

#[gdextension]
unsafe impl ExtensionLibrary for AirshipsGodot {}

/*#[derive(GodotClass)]
#[class(base=RigidBody3D)]
struct Player {
    speed: f32,
    throttle_rate: f32,
    control_torque: f32,

    drag_lin_local: nalg::Vector3<f32>,
    drag_ang_local: nalg::Vector3<f32>,

    base: Base<RigidBody3D>,
}

#[godot_api]
impl IRigidBody3D for Player {
    fn init(base: Base<RigidBody3D>) -> Self {
        godot_print!("Player initialized."); // Prints to the Godot console
        
        Self {
            speed: 100.0,
            throttle_rate: 100.0,
            control_torque: 50.0,
            drag_lin_local: nalg::vector![1., 1., 1.],
            drag_ang_local: nalg::vector![1., 1., 1.],
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let drag_ang_to_apply = self.drag_ang_local
            .component_mul(&vec3_godot2nalg(-self.base().get_angular_velocity()));
        self.base_mut().apply_torque(vec3_nalg2godot(drag_ang_to_apply));
    }
}*/

mod control_palette;
mod control_policy;
mod thruster;
mod balloon;
mod fin;
mod servo;
mod airship_controller;
mod ray_sensor_suite;

/* Module */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
