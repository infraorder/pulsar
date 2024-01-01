mod audio_graph;
mod dsp;
mod egui;
mod fps;
mod instancing;
mod line;
mod post;
mod util;

use std::ops::{Add, Mul};

use audio_graph::asset_reader::{CustomAssetLoader, LuaAsset};
use audio_graph::{Audio, AudioControl, AudioPlugin};
use bevy::app::PluginGroup;
use bevy::asset::{AssetPlugin, Assets};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::core_2d::{Camera2d, Camera2dBundle};
use bevy::core_pipeline::tonemapping::DebandDither;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::ecs::query::Without;
use bevy::ecs::system::ResMut;
use bevy::gizmos::gizmos::Gizmos;
use bevy::gizmos::GizmoConfig;
use bevy::log::{error, trace};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::SpatialBundle;
use bevy::render::camera::Camera;
use bevy::render::color::Color;
use bevy::render::mesh::{shape, Mesh};
use bevy::render::view::{Msaa, NoFrustumCulling, RenderLayers};
use bevy::sprite::Mesh2dHandle;
use bevy::time::Time;
use bevy::transform::components::GlobalTransform;
use bevy::{
    app::{Startup, Update},
    asset::{AssetApp, AssetServer, Handle},
    ecs::system::Query,
    prelude::{App, Commands, Res},
    DefaultPlugins,
};
use dsp::oscillators::Oscillator;
use dsp::read::Read;
use dsp::{Chain, Dsp};
use instancing::{InstanceData, InstanceMaterialData};
use line::{SplitLine, XYLine, SPLIT_LEN};
use post::bloom::BloomSettings;
use post::feedback::{FeedbackBundle, FeedbackSettings};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use util::{BASE, OVERLAY0, SURFACE1};

const SCALE_Z: f32 = 0.2;
const SCALE_Y: f32 = 50.;

const OFFSET_X_0: f32 = -0.97;
const OFFSET_Y_0: f32 = -0.8;

const OFFSET_X_1: f32 = -0.97;
const OFFSET_Y_1: f32 = -0.5;

const FREQUENCY: f32 = 244.;

#[cfg(debug_assertions)]
fn main() {
    use bevy::{
        app::PostUpdate,
        window::{PresentMode, Window, WindowPlugin, WindowResolution},
    };
    use bevy_egui::EguiPlugin;
    use fps::setup_fps_counter;

    use crate::{
        fps::{fps_counter_showhide, fps_text_update_system},
        instancing::InstanceMaterial2dPlugin,
        post::{bloom::BloomPlugin, feedback::FeedbackPlugin},
    };

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(1280., 720.),
                        title: "pulsar • player".into(),
                        present_mode: PresentMode::Immediate,
                        fit_canvas_to_parent: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            FrameTimeDiagnosticsPlugin::default(),
            BloomPlugin,
            FeedbackPlugin,
            EguiPlugin,
            InstanceMaterial2dPlugin,
            AudioPlugin,
        ))
        .insert_resource(Msaa::Sample8)
        .init_asset::<LuaAsset>()
        .init_asset_loader::<CustomAssetLoader>()
        .add_systems(Startup, (setup_sound, setup, setup_fps_counter))
        .add_systems(Update, change_frequency)
        .add_systems(Update, plot_out)
        .add_systems(Update, (oscil, line))
        .add_systems(Update, (fps_text_update_system, fps_counter_showhide))
        .add_systems(PostUpdate, clear_lines)
        .run()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut gizmo_config: ResMut<GizmoConfig>,
) {
    gizmo_config.render_layers = RenderLayers::layer(1);
    gizmo_config.line_width = 2.0;

    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
                ..Default::default()
            },
            camera: Camera {
                hdr: true,
                order: 1,
                ..Default::default()
            },
            ..Default::default()
        },
        // TODO: figure out a way to get this camer
        BloomSettings {
            intensity: 0.4,
            ..Default::default()
        },
        RenderLayers::from_layers(&[1]),
    ));

    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(BASE),
                ..Default::default()
            },
            camera: Camera {
                hdr: true,
                order: 0,
                ..Default::default()
            },
            deband_dither: DebandDither::Enabled,
            ..Default::default()
        },
        BloomSettings {
            intensity: 0.4,
            ..Default::default()
        },
        FeedbackBundle::default(),
        RenderLayers::from_layers(&[0]),
    ));

    commands.spawn(XYLine {
        ..Default::default()
    });

    commands.spawn(SplitLine {
        ..Default::default()
    });

    let mesh = meshes.add(Mesh::from(shape::Circle {
        radius: 1.5,
        ..Default::default()
    }));

    commands.spawn((
        Mesh2dHandle(mesh),
        SpatialBundle::INHERITED_IDENTITY,
        InstanceMaterialData(vec![]),
        NoFrustumCulling,
    ));
}

fn setup_sound(mut commands: Commands, asset_server: Res<AssetServer>) {
    let lua_handle: Handle<LuaAsset> = asset_server.load("lua/wave.lua");
    let lua_util_handle: Handle<LuaAsset> = asset_server.load("lua/util.lua");

    let oscil = Oscillator {
        frequency_hz: FREQUENCY,
        lua_handle,
        lua_util_handle,
        lua_string: "".to_owned(),
    };

    let read = Read {};

    let chain = Chain {
        items: vec![Dsp::Input(oscil), Dsp::Read(read)],
    };

    commands.spawn(Audio::new(chain));
}

fn change_frequency(q_control: Query<&AudioControl<Oscillator>>, time: Res<Time>) {
    if let Ok(control) = q_control.get_single() {
        trace!("TEST");
        trace!("FREQUENCY: {}", control.frequency());

        let exp = time.elapsed_seconds_wrapped().sin();
        let _frequency_hz = 2.0_f32.powf(exp) * FREQUENCY;
        control.set_frequency(_frequency_hz);
        control.set_time(time.delta_seconds());
    }
}

fn plot_out(
    q_control: Query<&AudioControl<Read>>,
    mut lines: Query<&mut XYLine>,
    mut split_lines: Query<&mut SplitLine>,
) {
    if let Ok(control) = q_control.get_single() {
        let last_out = control.last_out();

        if last_out.is_none() {
            return;
        }

        let mut split_line = split_lines.single_mut();
        let mut audio_line = lines.single_mut();

        match &last_out {
            Some(last_out) => {
                let end_idx = match last_out.0 >= (audio_line.buffer[0].len() - 1) {
                    true => {
                        error!("BUFFER OVERFLOW");
                        error!("BUFFER OVERFLOW");
                        error!("BUFFER OVERFLOW"); // TODO: find a better way to display that this is happening
                        error!("BUFFER OVERFLOW"); // as its a big fuck up when this happens
                        error!("BUFFER OVERFLOW");
                        audio_line.buffer[0].len() - 1
                    }
                    false => last_out.0,
                };

                audio_line
                    .buffer
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(i, line)| {
                        (0..end_idx).for_each(|j| {
                            line[j] = last_out.1[i][j];
                        });
                    });

                audio_line.index = end_idx;

                split_line
                    .buffer
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(i, line)| {
                        let idx = last_out.0;

                        if idx > SPLIT_LEN {
                            let idx = idx - SPLIT_LEN;
                            line.clear();
                            line.append(&mut last_out.1[i][idx..].to_vec());
                            return ();
                        }

                        if line.len() + idx > SPLIT_LEN {
                            line.drain(0..idx);
                        }

                        line.append(&mut last_out.1[i].to_vec())
                    });
            }
            None => (),
        }
    }
}

fn clear_lines(mut lines: Query<&mut XYLine>) {
    lines.iter_mut().for_each(|mut line| line.index = 0);
}

fn line(
    mut gizmos: Gizmos,
    camera_query: Query<(&Camera, &GlobalTransform), Without<FeedbackSettings>>,
    lines: Query<&SplitLine>,
) {
    let line = lines.single();
    let (camera, camera_transform) = camera_query.single();
    (0..line.buffer.len()).for_each(|i| {
        gizmos.linestrip_2d(
            to_vec2(camera, camera_transform, &line.buffer[i], &i),
            OVERLAY0.add(Color::rgb(2.0, 2.0, 2.0)),
        );
    });
}

fn oscil(lines: Query<&XYLine>, mut instance_material_data: Query<&mut InstanceMaterialData>) {
    let l = lines.single();
    let mut mat_data = instance_material_data.get_single_mut().unwrap();

    mat_data.0 = (0..l.index)
        .into_par_iter()
        .map(|i| InstanceData {
            position: Vec3::new(l.buffer[0][i] * 300., l.buffer[1][i] * 300., 0.0),
            scale: 1.0,
            index: 1.0,
            color: SURFACE1.mul(7.0).as_rgba_f32(),
        })
        .collect();
}

fn to_vec2(
    cam: &Camera,
    cam_tform: &GlobalTransform,
    last_out: &[f32],
    output: &usize,
) -> Vec<Vec2> {
    let offset = match output {
        0 => cam
            .ndc_to_world(cam_tform, Vec3::new(OFFSET_X_0, OFFSET_Y_0, 0.0))
            .unwrap(),
        1 => cam
            .ndc_to_world(cam_tform, Vec3::new(OFFSET_X_1, OFFSET_Y_1, 0.0))
            .unwrap(),
        _ => panic!("HOW DID WE GET HERE"),
    };

    last_out
        .iter()
        .rev()
        .enumerate()
        .map(|(i, sample)| {
            bevy::math::Vec2::new(
                (((i) as f32) * SCALE_Z) + offset.x,
                ((*sample) * SCALE_Y) + offset.y,
            )
        })
        .collect()
}
