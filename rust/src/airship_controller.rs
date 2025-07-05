use std::f32::consts::PI;

use godot::global::Key;
use godot::prelude::*;
use godot::classes::*;

use crate::utils::*;
use crate::control_palette::*;
use crate::control_policy::*;
use crate::thruster::*;

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct AirshipController {
    controls: ControlPalette,
    policy: ControlPolicy,

    #[export]
    throttle_rate: f32,

    #[var]
    forward_throttle: f32,
    #[var]
    turn_throttle: f32,
    #[var]
    vertical_throttle: f32,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for AirshipController {
    fn init(base: Base<Node3D>) -> Self {
        godot_print!("Balloon initialized."); // Prints to the Godot console

        Self {
            controls: ControlPalette::new(KEYS.to_vec()),
            policy: ControlPolicy{},
            throttle_rate: 0.02,
            forward_throttle: 0.0,
            turn_throttle: 0.,
            vertical_throttle: 4.,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        for key in self.controls.keys.clone() {
            let input = Input::singleton();
            self.controls.set_value(key, input.is_key_pressed(key));
            godot_print!("{}", self.controls.get_value(key));
        }

        let w = self.controls.get_value(KEYS[0].clone()) as i32;
        let a = self.controls.get_value(KEYS[1].clone()) as i32;
        let s = self.controls.get_value(KEYS[2].clone()) as i32;
        let d = self.controls.get_value(KEYS[3].clone()) as i32;
        let space = self.controls.get_value(KEYS[4].clone()) as i32;
        let ctrl = self.controls.get_value(KEYS[5].clone()) as i32;
        
        self.forward_throttle = self.forward_throttle - self.throttle_rate * ((w - s) as f32);
        self.turn_throttle = self.turn_throttle - self.throttle_rate * ((d - a) as f32);
        self.vertical_throttle = self.vertical_throttle - self.throttle_rate * ((space - ctrl) as f32);

        let throttle_right = self.forward_throttle - self.turn_throttle;
        let throttle_left = self.forward_throttle + self.turn_throttle;
        let throttle_bottomright = self.vertical_throttle;
        let throttle_bottomleft = self.vertical_throttle;

        let airship = self.base().get_child(0).unwrap(); // Physical Airship

        airship.get_child(1).unwrap().get_child(0).unwrap()
            .try_cast::<Thruster>().unwrap()
            .set("thrust", &throttle_right.to_variant());

        airship.get_child(2).unwrap().get_child(0).unwrap()
            .try_cast::<Thruster>().unwrap()
            .set("thrust", &throttle_left.to_variant());

        airship.get_child(3).unwrap().get_child(0).unwrap()
            .try_cast::<Thruster>().unwrap()
            .set("thrust", &throttle_bottomright.to_variant());

        airship.get_child(4).unwrap().get_child(0).unwrap()
            .try_cast::<Thruster>().unwrap()
            .set("thrust", &throttle_bottomleft.to_variant());


    }
}