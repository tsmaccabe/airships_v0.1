use std::f32::consts::PI;

use godot::prelude::*;
use godot::classes::*;

use crate::utils::*;

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct RaySensorSuite {
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
pub impl INode3D for RaySensorSuite {
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