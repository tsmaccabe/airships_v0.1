use std::any::Any;
use std::f32::consts::PI;
use std::str::FromStr;

use godot::classes::input_map;
use godot::global::Key;
use godot::meta::AsArg;
use godot::meta::AsObjectArg;
use godot::prelude::*;
use godot::classes::*;

use nalgebra::Rotation;
use nalgebra as nalg;


struct AirshipsGodot;

/* Utilities */

const KEYS: [Key; 6] = [Key::W, Key::A, Key::S, Key::D, Key::CTRL, Key::SPACE];

fn euler_to_dir_godot(euler: Vector3) -> Vector3 {
    let pitch = euler.x;  // Rotation around X-axis
    let yaw = euler.y;    // Rotation around Y-axis
    
    Vector3::new(
        -yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    )
}

fn quaternion_to_dir_godot(quat: Quaternion) -> Vector3 {
    let normalized_quat = quat.normalized();

    let forward = Vector3::new(0.0, 0.0, -1.0);

    normalized_quat * forward
}

/*fn euler_to_dir_godot(euler: Vector3) -> Vector3 {
    let pitch = euler.x;  // Rotation around X-axis
    let yaw = euler.y;    // Rotation around Y-axis
    
    Vector3::new(
        yaw.sin() * pitch.cos(),   // X component
        pitch.sin(),                // Y component  
        yaw.cos() * pitch.cos(),   // Z component
    )
} */

fn ensure_unique_keys(keys: &Vec<Key>) {
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

/*fn generate_ray_dirs(forward: Vector3, pitches: Vec<f32>, yaws: &mut Vec<f32>) -> Array<Vector3> {
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
}*/

/// Generates ray directions based on pitch and yaw inputs using quaternions.
fn generate_ray_dirs(forward: Vector3, pitches: Vec<f32>, yaws: &mut Vec<f32>) -> Array<Vector3> {
    let mut dirs = Array::new();

    for pitch in pitches.into_iter() {
        for yaw_ref in yaws.into_iter() {
            // Create a quaternion from pitch and yaw (roll is 0)
            let rotation = Quaternion::from_euler(Vector3::new(pitch, *yaw_ref, 0.0));

            // Rotate the forward vector using the quaternion
            let direction = rotation * forward;

            // Add the resulting direction to the array
            dirs.push(direction);
        }
    }

    dirs
}

/* Rust Classes */
struct ControlPalette {
    keys: Vec<Key>,
    pressed: Vec<bool>,
}

impl ControlPalette {
    fn new(keys: Vec<Key>) -> Self {
        ensure_unique_keys(&keys);

        let mut pressed = Vec::new();
        for _ in keys.iter() {
            pressed.push(false);
        }
        Self {keys, pressed}
    }

    fn get_value(&mut self, key: Key) -> bool {
        self.pressed[self.keys.iter().position(|r| *r == key).unwrap()]
    }

    fn get_values(&mut self, keys: Vec<Key>) -> Vec<bool> {
        ensure_unique_keys(&keys);

        let mut values = Vec::new();

        for key in keys {
            values.push(self.get_value(key));
        };
        values
    }

    fn set_value(&mut self, key: Key, pressed: bool) {
        if !self.keys.contains(&key) {
            panic!("ControlPalette: input key not found in this palette.")
        }
        self.pressed[self.keys.iter().position(|r| *r == key).unwrap()] = pressed;
    }

    fn set_values(&mut self, keys: Vec<Key>, pressed: Vec<bool>) {
        ensure_unique_keys(&keys);

        for (i, key) in keys.clone().into_iter().enumerate() {
            if !keys.contains(&key) {
                panic!("ControlPalette: input key not found in this palette.")
            }

            self.pressed[i] = pressed[i];
        };
    }
}

struct ControlPolicy {

}

impl ControlPolicy {
    fn control_signal(self, ) -> ControlPalette {
        //let input = 
        
        let mut keypress = Vec::new();
        //keypress.push("".to_string());
        ControlPalette::new(keypress)
    }
}

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

        let forward = euler_to_dir_godot(self.base().get_global_rotation());

        let thrust_vec = self.thrust * forward;
        godot_print!("thrust_vec {} {}: {}, {}, {}", self.base().get_name(), self.thrust, thrust_vec.x, thrust_vec.y, thrust_vec.z);
        
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

#[derive(GodotClass)]
#[class(base=Node3D)]
struct AirshipController {
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

#[derive(GodotClass)]
#[class(base=Node3D)]
struct RaySensorSuite {
    #[var]
    lengths: Array<f32>,
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

        let mut lengths = Array::new();

        let mut hit_positions = Array::new();
        let mut hit_normals = Array::new();
        let mut hit_bodies = Array::new();

        let mut pitches = Vec::new();
        let mut yaws = Vec::new();
        let n = 7;
        for i in 0..n {
                pitches.push(-PI/4. * (1. - 2.*(i as f32 + 0.5)/(n as f32)));
        }
        for i in 0..n {
                yaws.push(-PI/4. * (1. - 2.*(i as f32 + 0.5)/(n as f32)));
        }

        godot_print!("{:?}, {:?}", pitches, yaws);
        let ray_targets_local = generate_ray_dirs(Vector3::new(1000., 0., 0.), pitches, &mut yaws);
        
        for _ in 0..(n*n) {
            lengths.push(0.);
            hit_positions.push(Vector3::new(0., 0., 0.));
            hit_normals.push(Vector3::new(0., 0., 0.));
            hit_bodies.push(Vector3::new(0., 0., 0.));
        }

        Self {
            lengths,
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

            let norm_result = result.get("normal");
            match norm_result {
                Some(_) => {
                    self.hit_normals.set(i, norm_result.unwrap().try_to::<Vector3>().unwrap());
                }
                None => {
                    //godot_print!("nocollide");
                }
            }

            let pos_result = result.get("position");
            match pos_result {
                Some(_) => {
                    self.hit_positions.set(i, pos_result.unwrap().try_to::<Vector3>().unwrap());
                }
                None => {
                    //godot_print!("nocollide");
                }
            }

            for i in 1..self.lengths.len() {
                self.lengths.set(i, (self.base().get_global_position() - self.hit_positions.at(i)).length());
            }
            let lengths = self.lengths.clone();
            //godot_print!("{lengths}");
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
