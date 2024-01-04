pub mod system;

use bevy::{
    ecs::{bundle::Bundle, component::Component, entity::Entity},
    log::info,
    math::Vec3,
    utils::HashMap,
};

use crate::{instancing::InstanceData, util::MANTLE, InstancingBundle};

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

impl Grid {
    pub fn render_change(&self, data: &mut Vec<InstanceData>) {
        info!("render change");
        data.clear();

        let dim_x = self.dims.0;
        let dim_y = self.dims.1;

        data.append(
            &mut (0..dim_x)
                .into_iter()
                .map(|x| {
                    (0..dim_y)
                        .into_iter()
                        .map(|y| InstanceData {
                            position: Vec3::new(
                                ((x as f32) - (dim_x as f32 / 2.0)) * 10.0, // move to -1 1
                                ((y as f32) - (dim_y as f32 / 2.0)) * 10.0,
                                -2.0,
                            ),
                            scale: 1.0,
                            index: 0.0,
                            color: MANTLE.as_rgba_f32(),
                        })
                        .collect::<Vec<InstanceData>>()
                })
                .flatten()
                .collect::<Vec<InstanceData>>(),
        );
    }

    pub fn add_to_grid(&mut self, entity: Entity, pos: (u32, u32)) {
        // check if pos of grid x, y contains

        if self.map.contains_key(&pos) {
            // TODO: do something with this
            panic!("SHOULD not insert where data exists");
        }

        self.map.entry(pos).insert(entity);
    }

    pub fn remove_from_grid(&mut self, pos: (u32, u32)) {
        self.map.remove(&pos);
    }

    pub fn move_entity(&mut self, entity: Entity, pos: (u32, u32), npos: (u32, u32)) {
        self.remove_from_grid(pos);
        self.add_to_grid(entity, npos);
    }

    pub fn get_entities(&self, pos: (u32, u32)) -> Option<Entity> {
        self.map.get(&pos).cloned()
    }
}
