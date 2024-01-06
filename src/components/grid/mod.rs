pub mod system;

use bevy::{
    ecs::{bundle::Bundle, component::Component, entity::Entity},
    log::info,
    math::Vec3,
    utils::HashMap,
};

use anyhow::Result;

use crate::{instancing::InstanceData, util::MANTLE, InstancingBundle};

use super::nodes::types::NodeTrait;

#[derive(Debug, Component, Clone)]
pub struct Grid {
    pub dims: (i32, i32),
    pub map: HashMap<(i32, i32), Entity>,
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
    pub fn render_change(
        &self,
        data: &mut Vec<InstanceData>,
        widget_scale: f32,
        offset: (f32, f32),
    ) {
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
                                (((x as f32) - (dim_x as f32 / 2.0)) * offset.0) - (offset.0 / 2.0), // move to -1 1
                                (((y as f32) - (dim_y as f32 / 2.0)) * offset.1) - (offset.1 / 2.0),
                                -2.0,
                            ),
                            scale: widget_scale,
                            index: 0.0,
                            color: MANTLE.as_rgba_f32(),
                        })
                        .collect::<Vec<InstanceData>>()
                })
                .flatten()
                .collect::<Vec<InstanceData>>(),
        );
    }

    pub fn add_to_grid(&mut self, entity: Entity, pos: (i32, i32)) {
        // check if pos of grid x, y contains

        if self.map.contains_key(&pos) {
            // TODO: do something with this
            panic!("SHOULD not insert where data exists");
        }

        self.map.entry(pos).insert(entity);
    }

    pub fn remove_from_grid(&mut self, pos: (i32, i32)) {
        self.map.remove(&pos);
    }

    pub fn move_entity(&mut self, entity: Entity, pos: (i32, i32), npos: (i32, i32)) {
        self.remove_from_grid(pos);
        self.add_to_grid(entity, npos);
    }

    pub fn get_entities(&self, pos: (i32, i32)) -> Option<Entity> {
        self.map.get(&pos).cloned()
    }

    pub fn exists(&self, pos: (i32, i32)) -> bool {
        self.map.contains_key(&pos)
    }

    pub fn check_collision<
        T: NodeTrait + Component,
        S: NodeTrait + Component,
        V: NodeTrait + Component,
    >(
        &mut self,
        node: &T,
        input_slots: &Vec<S>,
        output_slots: &Vec<V>,
    ) -> Result<()> {
        if self.exists(node.pos().to_tuple()) {
            return Err(anyhow::anyhow!("collision"));
        }

        for slot in input_slots.iter() {
            if self.exists(slot.pos().offset(node.pos()).to_tuple()) {
                return Err(anyhow::anyhow!("collision"));
            }
        }

        for slot in output_slots.iter() {
            if self.exists(slot.pos().offset(node.pos()).to_tuple()) {
            return Err(anyhow::anyhow!("collision"));
            }
        }

        Ok(())
    }
}
