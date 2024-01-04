use bevy::{
    asset::Assets,
    ecs::{
        component::Component,
        system::{Commands, ResMut},
    },
    prelude::{Deref, DerefMut},
    render::{
        mesh::{Indices, Mesh},
        render_resource::PrimitiveTopology,
        view::RenderLayers,
    },
    time::Timer,
};

use crate::{instancing::InstanceMaterialData, InstancingBundle, UI_TARGET};

use super::{Grid, GridBundle};

pub fn setup_grid(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = bevy::sprite::Mesh2dHandle(meshes.add(construct_grid_mesh()));

    let mut instance_material = InstanceMaterialData {
        data: vec![],
        layer: RenderLayers::layer(UI_TARGET),
    };
    let grid = Grid::default();
    grid.render_change(&mut instance_material.data);

    commands.spawn((
        GridBundle {
            instancing: InstancingBundle {
                mesh,
                instance_material,
                ..Default::default()
            },
            grid,
            ..Default::default()
        },
        RenderLayers::layer(UI_TARGET),
    ));
}

#[derive(Component, Deref, DerefMut)]
pub struct GridAnimationTimer(Timer);

fn construct_grid_mesh() -> Mesh {
    let extent_x = 1.0 / 2.0;
    let extent_y = 2.0 / 2.0;

    let extent_2_x = 1.0 / 2.0;
    let extent_2_y = 1.0 / 2.0;

    let extent_3_x = 2.0 / 2.0;
    let extent_3_y = 1.0 / 2.0;

    /*
     *
     *
     *      1  2
     *
     *   6     4  7
     *
     *   5        8
     *
     *      0  3
     *
     */

    let (u_left, u_right) = (0.0, 1.0);
    let vertices = [
        ([-extent_x, -extent_y, 0.0], [0.0, 0.0, 1.0], [u_left, 1.0]),
        ([-extent_x, extent_y, 0.0], [0.0, 0.0, 1.0], [u_left, 0.0]),
        ([extent_x, extent_y, 0.0], [0.0, 0.0, 1.0], [u_right, 0.0]),
        ([extent_x, -extent_y, 0.0], [0.0, 0.0, 1.0], [u_right, 1.0]),
        (
            [extent_2_x, extent_2_y, 0.0],
            [0.0, 0.0, 1.0],
            [u_right, 0.0],
        ),
        (
            [-extent_3_x, -extent_3_y, 0.0],
            [0.0, 0.0, 1.0],
            [u_left, 1.0],
        ),
        (
            [-extent_3_x, extent_3_y, 0.0],
            [0.0, 0.0, 1.0],
            [u_left, 0.0],
        ),
        (
            [extent_3_x, extent_3_y, 0.0],
            [0.0, 0.0, 1.0],
            [u_right, 0.0],
        ),
        (
            [extent_3_x, -extent_3_y, 0.0],
            [0.0, 0.0, 1.0],
            [u_right, 1.0],
        ),
    ];

    let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2, 4, 7, 5, 8, 7, 6, 5]);

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    Mesh::new(PrimitiveTopology::TriangleList)
        .with_indices(Some(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
