use std::f32::consts::PI;

use godot::global::Key;
use godot::prelude::*;
use godot::classes::*;

use crate::utils::*;

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
pub struct Fin {
    #[export]
    pub fin_drag: f32,
    pub base: Base<RigidBody3D>,
}

#[godot_api]
impl IRigidBody3D for Fin {
    fn init(base: Base<RigidBody3D>) -> Self {
        godot_print!("Fin initialized."); // Prints to the Godot console
        
        Self {
            fin_drag: 1.,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let rot = self.base().get_global_rotation();
        let dir = euler_to_dir_godot(rot);
        let vel = self.base().get_linear_velocity();
        let drag = -self.fin_drag * dir.dot(vel) * dir;
        //godot_print!("{rot}, {dir}, {normal}, {drag}, {vel}");
        self.base_mut().apply_force(drag);
    }
}