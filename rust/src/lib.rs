use std::any::Any;
use std::f32::consts::PI;
use std::str::FromStr;

use godot::classes::input_map;
use godot::classes::light_3d::Param;
use godot::meta::AsArg;
use godot::meta::AsObjectArg;
use godot::prelude::*;
use godot::classes::*;

use godot::*;

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
        yaw.sin() * pitch.cos(),   // X component
        pitch.sin(),                // Y component  
        yaw.cos() * pitch.cos(),   // Z component
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

fn generate_ray_dirs(forward: Vector3, pitches: Vec<f32>, yaws: &mut Vec<f32>) -> Array<Vector3> {
    let mut dirs = Array::new();
    for pitch in pitches.into_iter() {
        for yaw_ref in yaws.into_iter() {
            let euler = Vector3::new(0., *yaw_ref, pitch);

            dirs.push(forward
                .rotated(Vector3::new(0., 1., 0.), euler.x)
                .rotated(Vector3::new(1., 0., 0.), euler.y)
                .rotated(Vector3::new(0., 0., 1.), euler.z));
        }
    }
    dirs
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

#[derive(Clone, Debug)]
struct ControlPolicy {

}

impl ControlPolicy {
    fn control_signal(self) -> ControlPalette {
        let mut keypress = Vec::new();
        //keypress.push("".to_string());
        ControlPalette::new(keypress)
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

        let forward = euler_to_dir_godot(self.base().get_global_rotation() + Vector3::new(0., 0., 0.));

        let thrust_vec = self.thrust * forward;
        //godot_print!("thrust_forward {}: {}, {}, {}", self.base().get_name(), forward.x, forward.y, forward.z);
        
        self.base_mut().apply_force(thrust_vec);
    }
}

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
struct Balloon {
    #[export]
    bouyancy: Vector3,
    base: Base<RigidBody3D>,
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

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
struct Fin {
    #[export]
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

        let d_angle = self.target_angle - (godot::global::sqrt((self.base().get_rotation().x.powf(2.) + self.base().get_rotation().y.powf(2.) + self.base().get_rotation().z.powf(2.)) as f64) as f32);

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

        airship.get_child(1).unwrap().get_child(0).unwrap()
            .set_meta("Thrust", &throttle_left.to_variant()); // Right Thruster

        airship.get_child(2).unwrap().get_child(0).unwrap()
            .set_meta("Thrust", &throttle_right.to_variant()); // Left Thruster

        airship.get_child(3).unwrap().get_child(0).unwrap()
            .set_meta("Thrust", &throttle_bottom.to_variant()); // Bottom Thruster
    }
}

#[derive(GodotClass)]
#[class(base=Node3D)]
struct RaySensorSuite {
    #[var]
    ray_targets_local: Array<Vector3>,
    #[var]
    hit_positions: Array<Vector3>,
    #[var]
    hit_normals: Array<Vector3>,
    #[var]
    hit_bodies: Array<Vector3>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RaySensorSuite {
    fn init(base: Base<Node3D>) -> Self {
        godot_print!("RaySensorSuite initialized."); // Prints to the Godot console

        let mut hit_positions = Array::new();

        let mut hit_normals = Array::new();

        let mut hit_bodies = Array::new();

        let mut pitches = Vec::new();
        let mut yaws = Vec::new();
        let n = 3;
        for i in 0..n {
                pitches.push(-PI/2. * (1. + 2. * (i as f32)/(n as f32 + 1.)));
        }
        for i in 0..n {
                yaws.push(-PI/2. * (1. + 2. * (i as f32)/(n as f32 + 1.)));
        }

        let ray_targets_local = generate_ray_dirs(Vector3::new(1000., 0., 0.), pitches, &mut yaws);
        
        for _ in 0..(n*n) {
            hit_positions.push(Vector3::new(0., 0., 0.));
            hit_normals.push(Vector3::new(0., 0., 0.));
            hit_bodies.push(Vector3::new(0., 0., 0.));
        }

        Self {
            ray_targets_local,
            hit_positions,
            hit_normals,
            hit_bodies,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let mut space_state = self.base()
            .get_world_3d().unwrap()
            .try_cast::<World3D>().unwrap()
            .get_direct_space_state().unwrap();

        let sensor_local_to_global = self.base().get_global_transform().affine_inverse();
        let sensor_local_to_global_eulers = sensor_local_to_global.basis.get_euler();
        let ray_origin_pos_global = self.base().get_global_position();
        
        for i in 0..self.ray_targets_local.len() {
            let target_global = self.ray_targets_local.at(i)
                .rotated(Vector3::new(0., 1., 0.), -sensor_local_to_global_eulers.y)
                .rotated(Vector3::new(1., 0., 0.), -sensor_local_to_global_eulers.x)
                .rotated(Vector3::new(0., 0., 1.), -sensor_local_to_global_eulers.z) - sensor_local_to_global.origin;
            
            let result = space_state.intersect_ray(
                &PhysicsRayQueryParameters3D::create(ray_origin_pos_global, target_global).unwrap()
            );

            let pos_result = result.get("position");
            match pos_result {
                Some(_) => {
                    self.hit_positions.set(i, pos_result.unwrap().try_to::<Vector3>().unwrap());
                }
                None => {
                    godot_print!("nocollide");
                }
            }

            let norm_result = result.get("normal");
            match norm_result {
                Some(_) => {
                    match norm_result.unwrap().try_to::<Vector3>() {
                        Ok(norm) => {
                            //godot_print!("{pos}");
                            self.hit_normals.set(i, norm);
                        }
                        Err(_) => {
                            godot_print!("nocollide");
                        }
                    }
                }
                None => {
                    godot_print!("nocollide");
                }
            }
        }
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
