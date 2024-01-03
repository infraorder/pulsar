use bevy::{
    asset::{Asset, Handle},
    ecs::{component::Component, system::Resource},
    reflect::TypePath,
};
use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Debug, Deserialize, Serialize, Resource)]
pub struct ConfigAsset {
    // WINDOW CONFIGS
    pub width: u32,
    pub height: u32,
    // \ WINDOW_CONFIGS

    // LINE
    pub line_width: f32,
    pub line_scale_z: f32,
    pub line_scale_y: f32,

    pub line_offset_x_0: f32,
    pub line_offset_y_0: f32,

    pub line_offset_x_1: f32,
    pub line_offset_y_1: f32,
    // \ LINE

    // XY
    pub xy_mult: f32,
    pub xy_rad: f32,
    // \ XY

    // OSCIL - TEMP
    pub frequency: f32,
}

#[derive(Default, Component)]
pub struct ConfigComp {
    pub handle: Handle<ConfigAsset>,
}

// pub fn map_resmut_config_resource(config: &mut ResMut<ConfigResource>, new_config: &ConfigAsset) {
//     config.width = new_config.width;
//     config.height = new_config.height;
//     config.line_width = new_config.line_width;
//     config.line_scale_z = new_config.line_scale_z;
//     config.line_scale_y = new_config.line_scale_y;
//     config.line_offset_x_0 = new_config.line_offset_x_0;
//     config.line_offset_y_0 = new_config.line_offset_y_0;
//     config.line_offset_x_1 = new_config.line_offset_x_1;
//     config.line_offset_y_1 = new_config.line_offset_y_1;
//     config.xy_mult = new_config.xy_mult;
//     config.xy_rad = new_config.xy_rad;
//     config.frequency = new_config.frequency;
// }

pub fn map_config_resource(config: &mut ConfigAsset, new_config: &ConfigAsset) {
    config.width = new_config.width;
    config.height = new_config.height;
    config.line_width = new_config.line_width;
    config.line_scale_z = new_config.line_scale_z;
    config.line_scale_y = new_config.line_scale_y;
    config.line_offset_x_0 = new_config.line_offset_x_0;
    config.line_offset_y_0 = new_config.line_offset_y_0;
    config.line_offset_x_1 = new_config.line_offset_x_1;
    config.line_offset_y_1 = new_config.line_offset_y_1;
    config.xy_mult = new_config.xy_mult;
    config.xy_rad = new_config.xy_rad;
    config.frequency = new_config.frequency;
}
