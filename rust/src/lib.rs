use std::any::Any;
use std::f32::consts::PI;
use std::str::FromStr;

use godot::classes::input_map;
use godot::classes::light_3d::Param;
use godot::classes::Control;
use godot::prelude::*;
use godot::classes::RigidBody3D;
use godot::classes::IRigidBody3D;
use godot::classes::HingeJoint3D;
use godot::classes::IHingeJoint3D;
use godot::classes::Node3D;
use godot::classes::INode3D;
use godot::classes::Marker3D;
use godot::classes::IMarker3D;
use godot::classes::MeshInstance3D;
use godot::classes::IMeshInstance3D;

use godot::*;

use godot::global::sqrt;

use nalgebra::Rotation;
use nalgebra as nalg;

struct AirshipsGodot;

/* Utilities */

const KEYS: [&str; 6] = ["w", "a", "s", "d", "ctrl", "space"];

fn keys_string() -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for key in KEYS {
        keys.push(key.to_string());
    }
    keys
}


fn vec3_godot2nalg(v: Vector3) -> nalg::Vector3<f32> {
    nalg::vector![v.x, v.y, v.z]
}

fn vec3_nalg2godot(v: nalg::Vector3<f32>) -> Vector3 {
    Vector3::new(v.x, v.y, v.z)
}


fn euler_to_dir_godot(euler: Vector3) -> Vector3 {
    let pitch = euler.x;  // Rotation around X-axis
    let yaw = euler.y;    // Rotation around Y-axis
    
    Vector3::new(
        -yaw.sin() * pitch.cos(),   // X component
        pitch.sin(),                // Y component  
        -yaw.cos() * pitch.cos(),   // Z component
    )
}

fn ensure_unique_keys(keys: &Vec<String>) {
    for (i, key1) in keys.clone().into_iter().enumerate() {
        for (j, key2) in keys.clone().into_iter().enumerate() {
            if i == j {
                continue
            } else if key1 == key2 {
                panic!("ControlPalette: each element of `keys` must be unique.")
            }
        }
    }
}


/* Rust Classes */

#[derive(Clone, Debug)]
struct ControlPalette {
    keys: Vec<String>,
    pressed: Vec<bool>,
}

impl ControlPalette {
    fn new(keys: Vec<String>) -> Self {
        ensure_unique_keys(&keys);

        let mut pressed = Vec::new();
        for _ in keys.iter() {
            pressed.push(false);
        }
        Self {keys, pressed}
    }

    fn get_value(&mut self, key: String) -> bool {
        self.pressed[self.keys.iter().position(|r| *r == key).unwrap()]
    }

    fn get_values(&mut self, keys: Vec<String>) -> Vec<bool> {
        ensure_unique_keys(&keys);

        let mut values = Vec::new();

        for key in keys {
            values.push(self.get_value(key));
        };
        values
    }

    fn set_value(&mut self, key: String, pressed: bool) {
        if !self.keys.contains(&key) {
            panic!("ControlPalette: input key not found in this palette.")
        }
        self.pressed[self.keys.iter().position(|r| *r == key).unwrap()] = pressed;
    }

    fn set_values(&mut self, keys: Vec<String>, pressed: Vec<bool>) {
        ensure_unique_keys(&keys);

        for (i, key) in keys.clone().into_iter().enumerate() {
            if !keys.contains(&key) {
                panic!("ControlPalette: input key not found in this palette.")
            }

            self.pressed[i] = pressed[i];
        };
    }
}

/* Godot Classes */

#[gdextension]
unsafe impl ExtensionLibrary for AirshipsGodot {}

#[derive(GodotClass)]
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
}

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
struct Thruster {
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

        let forward = euler_to_dir_godot(self.base().get_global_rotation() + Vector3::new(0., 0., 0.));

        let thrust_vec = self.thrust * forward;
        self.thrust = self.base().get_meta("Thrust").to();
        godot_print!("thrust_forward {}: {}, {}, {}", self.base().get_name(), forward.x, forward.y, forward.z);
        
        self.base_mut().apply_force(thrust_vec);
    }
}

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
struct Balloon {
    bouyancy: nalg::Vector3<f32>,
    base: Base<RigidBody3D>,
}

#[godot_api]
impl IRigidBody3D for Balloon {
    fn init(base: Base<RigidBody3D>) -> Self {
        godot_print!("Balloon initialized."); // Prints to the Godot console
        
        Self {
            bouyancy: nalg::vector![0., 19., 0.],
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let bouyancy = self.bouyancy.clone();
        self.base_mut().apply_force(vec3_nalg2godot(bouyancy));
    }
}

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
struct Fin {
    fin_drag: f32,
    base: Base<RigidBody3D>,
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

#[derive(GodotClass)]
#[class(base=HingeJoint3D)]
struct Servo {
    constant: f32,
    damping: f32,
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

        let d_angle = self.target_angle - (sqrt((self.base().get_rotation().x.powf(2.) + self.base().get_rotation().y.powf(2.) + self.base().get_rotation().z.powf(2.)) as f64) as f32);

        self.base_mut().set_param(godot::classes::hinge_joint_3d::Param::MOTOR_TARGET_VELOCITY, d_angle*constant);
    }
}

#[derive(GodotClass)]
#[class(base=Node3D)]
struct AirshipController {
    controls: ControlPalette,

    throttle_rate: f32,
    forward_throttle: f32,
    turn_throttle: f32,
    vertical_throttle: f32,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for AirshipController {
    fn init(base: Base<Node3D>) -> Self {
        godot_print!("Balloon initialized."); // Prints to the Godot console

        Self {
            controls: ControlPalette::new(keys_string()),
            throttle_rate: 0.1,
            forward_throttle: 0.5,
            turn_throttle: 0.,
            vertical_throttle: 0.5,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let keys = keys_string();

        self.forward_throttle = self.forward_throttle + self.throttle_rate * ((self.controls.get_value(keys[0].clone()) as i32 - self.controls.get_value(keys[2].clone()) as i32) as f32);

        self.turn_throttle = self.turn_throttle + self.throttle_rate * ((self.controls.get_value(keys[3].clone()) as i32 - self.controls.get_value(keys[1].clone()) as i32) as f32);
        
        self.vertical_throttle = self.vertical_throttle + self.throttle_rate * ((self.controls.get_value(keys[5].clone()) as i32 - self.controls.get_value(keys[4].clone()) as i32) as f32);

        let throttle_right = self.forward_throttle - self.turn_throttle;
        let throttle_left = self.forward_throttle + self.turn_throttle;
        let throttle_bottom = self.vertical_throttle;

        let airship = self.base().get_child(0).unwrap(); // Physical Airship

        airship.get_child(1).unwrap()
            .set_meta("Thrust", &throttle_left.to_variant()); // Right Thruster

        airship.get_child(2).unwrap()
            .set_meta("Thrust", &throttle_right.to_variant()); // Left Thruster

        airship.get_child(3).unwrap()
            .set_meta("Thrust", &throttle_bottom.to_variant()); // Bottom Thruster
    }
}


/* Module */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
