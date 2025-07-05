use std::f32::consts::PI;

use godot::global::Key;
use godot::prelude::*;
use godot::classes::*;

pub const KEYS: [Key; 6] = [Key::W, Key::A, Key::S, Key::D, Key::CTRL, Key::SPACE];

pub fn euler_to_dir_godot(euler: Vector3) -> Vector3 {
    let pitch = euler.x;  // Rotation around X-axis
    let yaw = euler.y;    // Rotation around Y-axis
    
    Vector3::new(
        -yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    )
}

pub fn quaternion_to_dir_godot(quat: Quaternion) -> Vector3 {
    let normalized_quat = quat.normalized();

    let forward = Vector3::new(0.0, 0.0, -1.0);

    normalized_quat * forward
}

/*pub fn euler_to_dir_godot(euler: Vector3) -> Vector3 {
    let pitch = euler.x;  // Rotation around X-axis
    let yaw = euler.y;    // Rotation around Y-axis
    
    Vector3::new(
        yaw.sin() * pitch.cos(),   // X component
        pitch.sin(),                // Y component  
        yaw.cos() * pitch.cos(),   // Z component
    )
} */

pub fn ensure_unique_keys(keys: &Vec<Key>) {
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

/*pub fn generate_ray_dirs(forward: Vector3, pitches: Vec<f32>, yaws: &mut Vec<f32>) -> Array<Vector3> {
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
pub fn generate_ray_dirs(forward: Vector3, pitches: Vec<f32>, yaws: &mut Vec<f32>) -> Array<Vector3> {
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