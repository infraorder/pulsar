pub mod system;

use bevy::{
    ecs::{bundle::Bundle, component::Component, entity::Entity},
    math::Vec3,
    utils::HashMap,
};

use crate::{
    instancing::{InstanceData, InstanceMaterialData},
    util::MANTLE,
    InstancingBundle,
};

#[derive(Debug, Component, Clone)]
pub struct Grid {
    dims: (u32, u32),
    map: HashMap<(u32, u32), Entity>,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            dims: (64, 64),
            map: HashMap::default(),
        }
    }
}

#[derive(Bundle, Default)]
pub struct GridBundle {
    pub grid: Grid,
    pub instancing: InstancingBundle,
}

pub fn render_change(grid: &mut Grid, data: &mut InstanceMaterialData) {
    data.data.clear();
    data.data = (0..grid.dims.0)
        .into_iter()
        .map(|x| {
            (0..grid.dims.1)
                .into_iter()
                .map(|y| InstanceData {
                    position: Vec3::new(
                        (x - grid.dims.0) as f32 * 10.0, // move to -1 1
                        (y - grid.dims.1) as f32 * 10.0,
                        -2.0,
                    ),
                    scale: 1.0,
                    index: 0.0,
                    color: MANTLE.as_rgba_f32(),
                })
                .collect::<Vec<InstanceData>>()
        })
        .flatten()
        .collect();
}

pub fn add_to_grid(grid: &mut Grid, entity: Entity, pos: (u32, u32)) {
    // check if pos of grid x, y contains

    if grid.map.contains_key(&pos) {
        // TODO: do something with this
        panic!("SHOULD not insert where data exists");
    }

    grid.map.entry(pos).insert(entity);
}

pub fn remove_from_grid(grid: &mut Grid, pos: (u32, u32)) {
    grid.map.remove(&pos);
}

pub fn move_entity(grid: &mut Grid, entity: Entity, pos: (u32, u32), npos: (u32, u32)) {
    remove_from_grid(grid, pos);
    add_to_grid(grid, entity, npos);
}

pub fn get_entities(grid: &Grid, pos: (u32, u32)) -> Option<Entity> {
    grid.map.get(&pos).cloned()
}
