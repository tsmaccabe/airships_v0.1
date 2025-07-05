use std::f32::consts::PI;

use godot::global::Key;
use godot::prelude::*;
use godot::classes::*;

use crate::utils::*;
use crate::control_palette::*;

pub struct ControlPolicy {

}

impl ControlPolicy {
    fn control_signal(self, ) -> ControlPalette {
        //let input = 
        
        let mut keypress = Vec::new();
        //keypress.push("".to_string());
        ControlPalette::new(keypress)
    }
}