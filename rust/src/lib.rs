use std::any::Any;
use std::f32::consts::PI;
use std::str::FromStr;
use std::ops::Deref;

use godot::classes::input_map;
use godot::classes::light_3d::Param;
use godot::meta::AsArg;
use godot::meta::AsObjectArg;
use godot::prelude::*;
use godot::classes::*;
use godot::builtin::*; 

use godot::*;

use godot::global::sqrt;

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

fn generate_ray_dirs(forward: Vector3, pitches: Vec<f32>, yaws: &mut Vec<f32>) -> Vec<Vector3> {
    let mut dirs = Vec::new();
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
        //godot_print!("thrust_forward {}: {}, {}, {}", self.base().get_name(), forward.x, forward.y, forward.z);
        
        self.base_mut().apply_force(thrust_vec);
    }
}

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
struct Balloon {
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
        let mut ray_targets_local = Array::new();
        ray_targets_local.push(Vector3::new(0., -100., 0.));
        
        let mut hit_positions = Array::new();
        hit_positions.push(Vector3::new(0., 0., 0.));
        
        let mut hit_normals = Array::new();
        hit_normals.push(Vector3::new(0., 0., 0.));

        let mut hit_bodies = Array::new();
        hit_bodies.push(Vector3::new(0., 0., 0.));

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

            match result.get("position").unwrap().try_to::<Vector3>() {
                Ok(pos) => {
                    godot_print!("{pos}");
                    self.hit_positions.set(i, pos);
                }
                Err(_) => {
                    godot_print!("nocollide");
                    // No collision
                }
            }
        }
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
        godot_print!("AirshipController initialized."); // Prints to the Godot console

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

        airship.get_child(0).unwrap()
            .set_meta("Thrust", &throttle_left.to_variant()); // Right Thruster

        airship.get_child(1).unwrap()
            .set_meta("Thrust", &throttle_right.to_variant()); // Left Thruster

        airship.get_child(2).unwrap()
            .set_meta("Thrust", &throttle_bottom.to_variant()); // Bottom Thruster

        let mut marker = airship.get_child(3).unwrap().try_cast::<MeshInstance3D>().unwrap();
        let children = airship.get_children();
        //godot_print!("{children}");
        let binding = airship.get_child(1).unwrap().get_child(0).unwrap().get_child(2).unwrap().try_cast::<RaySensorSuite>().unwrap();
        let binding = binding.bind();
        let rays_front = binding.deref();

        
        let binding = airship.get_child(2).unwrap().get_child(0).unwrap().get_child(2).unwrap().try_cast::<RaySensorSuite>().unwrap();
        let binding = binding.bind();
        let rays_bottom = binding.deref();

        let hit_position = rays_front.hit_positions.at(0);

        //godot_print!("{hit_position}");
        marker.set_global_position(hit_position);
    }
}

/* Behavior Function */

fn update_controls(vehicle: &mut AirshipController, controls: &mut ControlPalette) {
    
}

/* Module */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
