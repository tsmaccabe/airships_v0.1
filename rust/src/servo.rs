use std::f32::consts::PI;

use godot::global::Key;
use godot::prelude::*;
use godot::classes::*;

use crate::utils::*;

#[derive(GodotClass)]
#[class(base=HingeJoint3D)]
pub struct Servo {
    #[export]
    constant: f32,
    #[export]
    damping: f32,
    #[export]
    target_angle: f32,
    base: Base<HingeJoint3D>,
}

#[godot_api]
impl IHingeJoint3D for Servo {
    fn init(base: Base<HingeJoint3D>) -> Self {
        godot_print!("Hinge initialized."); // Prints to the Godot console
        Self {
            constant: 100.,
            damping: 10.,
            target_angle: PI/6., // Degrees
            base
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let delta32 = delta as f32;

        let constant = self.constant;

        let d_angle = self.target_angle - (godot::global::sqrt((self.base().get_rotation().x.powf(2.) + self.base().get_rotation().y.powf(2.) + self.base().get_rotation().z.powf(2.)) as f64) as f32);

        self.base_mut().set_param(godot::classes::hinge_joint_3d::Param::MOTOR_TARGET_VELOCITY, d_angle*constant);
    }
}