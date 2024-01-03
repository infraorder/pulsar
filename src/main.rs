mod components;
mod dsp;
mod egui;
mod instancing;
mod lua;
mod post;
mod systems;
mod util;

use std::ops::Mul;

use bevy::app::PluginGroup;
use bevy::asset::{AssetEvent, AssetPlugin, Assets};
use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomSettings};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::core_2d::{Camera2d, Camera2dBundle};
use bevy::core_pipeline::tonemapping::DebandDither;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::event::EventReader;
use bevy::ecs::query::With;
use bevy::ecs::system::ResMut;
use bevy::gizmos::gizmos::Gizmos;
use bevy::gizmos::GizmoConfig;
use bevy::log::{error, trace};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::SpatialBundle;
use bevy::reflect::Reflect;
use bevy::render::camera::Camera;
use bevy::render::color::Color;
use bevy::render::mesh::{shape, Mesh};
use bevy::render::view::{Msaa, NoFrustumCulling, RenderLayers};
use bevy::sprite::{Mesh2dHandle, SpriteSheetBundle, TextureAtlas, TextureAtlasSprite};
use bevy::time::Time;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::{
    app::{Startup, Update},
    asset::{AssetApp, AssetServer, Handle},
    ecs::system::Query,
    prelude::{App, Commands, Res},
    DefaultPlugins,
};
use components::config::{map_config_resource, ConfigAsset, ConfigComp};
use components::line::{SplitLine, XYLine, SPLIT_LEN};
use components::lua::LuaAsset;
use dsp::audio_graph::{Audio, AudioControl};
use dsp::oscillators::Oscillator;
use dsp::read::Read;
use dsp::{Chain, Dsp};
use instancing::{InstanceData, InstanceMaterialData};
use post::feedback::FeedbackBundle;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use util::{BASE, OVERLAY0};

const OSCIL_TARGET: u8 = 1;
const UI_TARGET: u8 = 0;

const FREQUENCY_TEMP: f32 = 144.0;

#[derive(Bundle, Default)]
pub struct InstancingBundle {
    mesh: Mesh2dHandle,
    spatial: SpatialBundle,
    instance_material: InstanceMaterialData,
    frustum_culling: NoFrustumCulling,
}

#[cfg(debug_assertions)]
fn main() {
    use std::fs;

    use bevy::{
        app::PostUpdate,
        ecs::schedule::IntoSystemConfigs,
        render::texture::ImagePlugin,
        window::{PresentMode, Window, WindowPlugin, WindowResolution, WindowMode},
    };
    use bevy_egui::EguiPlugin;

    use crate::{
        components::{config::ConfigLoader, lua::LuaLoader},
        dsp::audio_graph::AudioPlugin,
        instancing::InstanceMaterial2dPlugin,
        post::feedback::FeedbackPlugin,
        systems::fps::{fps_counter_showhide, fps_text_update_system, setup_fps_counter},
    };

    let s = fs::read_to_string("assets/config.toml");
    let config = toml::from_str::<ConfigAsset>(&s.unwrap()).unwrap();

    println!("CONFIG: {:#?}", config);

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(
                            config.width as f32,
                            config.height as f32,
                        ),
                        title: "pulsar • player".into(),
                        mode: WindowMode::BorderlessFullscreen,
                        present_mode: PresentMode::Fifo,
                        fit_canvas_to_parent: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            FrameTimeDiagnosticsPlugin::default(),
            FeedbackPlugin,
            EguiPlugin,
            InstanceMaterial2dPlugin,
            AudioPlugin,
        ))
        .insert_resource(Msaa::Sample8)
        .insert_resource(config)
        .init_asset::<LuaAsset>()
        .init_asset_loader::<LuaLoader>()
        .init_asset::<ConfigAsset>()
        .init_asset_loader::<ConfigLoader>()
        // setup
        .add_systems(Startup, (setup, setup_fps_counter))
        // temporary setup will be removed in future
        .add_systems(Startup, setup_temp)
        // temporary system
        .add_systems(Update, change_frequency)
        // main drawing systems
        .add_systems(Update, (plot_out, (oscil, line)).chain())
        .add_systems(PostUpdate, clear_lines)
        // fps counter systems
        .add_systems(Update, (fps_text_update_system, fps_counter_showhide))
        // update config
        .add_systems(PostUpdate, update_config)
        .run()
}

fn setup(
    mut commands: Commands,
    mut gizmo_config: ResMut<GizmoConfig>,
    config: Res<ConfigAsset>,
    asset_server: Res<AssetServer>,
) {
    gizmo_config.render_layers = RenderLayers::layer(UI_TARGET);
    gizmo_config.line_width = config.line_width;

    let handle = asset_server.load("config.toml");

    commands.spawn(ConfigComp { handle });

    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
                ..Default::default()
            },
            camera: Camera {
                hdr: true,
                order: -1,
                ..Default::default()
            },
            deband_dither: DebandDither::Enabled,
            ..Default::default()
        },
        BloomSettings {
            intensity: 0.25,
            composite_mode: BloomCompositeMode::Additive,
            high_pass_frequency: 2.0,
            ..Default::default()
        },
        RenderLayers::layer(UI_TARGET),
        UICamera,
    ));

    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(BASE),
                ..Default::default()
            },
            camera: Camera {
                hdr: true,
                order: -2,
                ..Default::default()
            },
            deband_dither: DebandDither::Enabled,
            ..Default::default()
        },
        FeedbackBundle::default(),
        RenderLayers::layer(OSCIL_TARGET),
        OscilCamera,
    ));
}

// TODO: properly init env
fn setup_temp(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let lua_handle: Handle<LuaAsset> = asset_server.load("lua/readers/lua_pulse/wave.lua");
    let lua_util_handle: Handle<LuaAsset> = asset_server.load("lua/readers/lua_pulse/hood.lua");

    let oscil = Oscillator {
        frequency_hz: FREQUENCY_TEMP,
        lua_handle,
        lua_util_handle,
        lua_string: "".to_owned(),
    };

    let read = Read {};

    let chain = Chain {
        items: vec![Dsp::Input(oscil), Dsp::Read(read)],
    };

    commands.spawn(Audio::new(chain));

    commands.spawn(XYLine {
        ..Default::default()
    });

    commands.spawn(SplitLine {
        ..Default::default()
    });

    let mesh = meshes.add(Mesh::from(shape::Circle {
        radius: 1.0,
        ..Default::default()
    }));

    let texture_handle = asset_server.load("sprites/sheet.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(7., 9.),
        2,
        1,
        None,
        Some(Vec2::new(14., 0.)),
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let spundle = CustomSpriteBundle {
        texture_atlas: texture_atlas_handle,
        sprite: TextureAtlasSprite::new(1),
        ..Default::default()
    };

    // Use only the subset of sprites in the sheet that make up the run animation

    commands.spawn((InstancingBundle {
        mesh: Mesh2dHandle(mesh),
        frustum_culling: NoFrustumCulling,
        spatial: SpatialBundle::INHERITED_IDENTITY,
        instance_material: InstanceMaterialData {
            data: vec![InstanceData {
                color: Color::WHITE.as_linear_rgba_f32(),
                index: 1.0,
                position: Vec3::new(20.0, 0.0, 0.0),
                scale: 1.0,
            }],
            layer: RenderLayers::layer(OSCIL_TARGET),
        },
    },));
}

#[derive(Bundle, Clone, Default)]
pub struct CustomSpriteBundle {
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
}

fn change_frequency(q_control: Query<&AudioControl<Oscillator>>, time: Res<Time>) {
    if let Ok(control) = q_control.get_single() {
        trace!("TEST");
        trace!("FREQUENCY: {}", control.frequency());

        let exp = time.elapsed_seconds_wrapped().sin();
        let _frequency_hz = 2.0_f32.powf(exp) * FREQUENCY_TEMP;
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
    camera_query: Query<(&Camera, &GlobalTransform), With<UICamera>>,
    lines: Query<&SplitLine>,
    config: Res<ConfigAsset>,
) {
    let line = lines.single();
    let (camera, camera_transform) = camera_query.single();
    (0..line.buffer.len()).for_each(|i| {
        gizmos.linestrip_2d(
            to_vec2(camera, camera_transform, &line.buffer[i], &i, &config),
            OVERLAY0.mul(3.0),
        );
    });
}

fn oscil(
    lines: Query<&XYLine>,
    mut instance_material_data: Query<&mut InstanceMaterialData>,
    config: Res<ConfigAsset>,
) {
    let l = lines.single();
    let mut mat_data = instance_material_data.get_single_mut().unwrap();

    mat_data.data = (0..l.index)
        .into_par_iter()
        .map(|i| InstanceData {
            position: Vec3::new(
                l.buffer[0][i] * config.xy_mult,
                l.buffer[1][i] * config.xy_mult,
                0.0,
            ),
            scale: config.xy_rad,
            index: 1.0,
            color: OVERLAY0.as_rgba_f32(),
        })
        .collect();
}

fn to_vec2(
    cam: &Camera,
    cam_tform: &GlobalTransform,
    last_out: &[f32],
    output: &usize,
    config: &ConfigAsset,
) -> Vec<Vec2> {
    let offset = match output {
        0 => cam
            .ndc_to_world(
                cam_tform,
                Vec3::new(config.line_offset_x_0, config.line_offset_y_0, 0.0),
            )
            .unwrap(),
        1 => cam
            .ndc_to_world(
                cam_tform,
                Vec3::new(config.line_offset_x_1, config.line_offset_y_1, 0.0),
            )
            .unwrap(),
        _ => panic!("HOW DID WE GET HERE"),
    };

    last_out
        .iter()
        .rev()
        .enumerate()
        .map(|(i, sample)| {
            bevy::math::Vec2::new(
                (((i) as f32) * config.line_scale_z) + offset.x,
                ((*sample) * config.line_scale_y) + offset.y,
            )
        })
        .collect()
}

fn update_config(
    mut config_event: EventReader<AssetEvent<ConfigAsset>>,
    config_assets: Res<Assets<ConfigAsset>>,
    mut config: ResMut<ConfigAsset>,
) {
    for ev in config_event.read() {
        match ev {
            AssetEvent::Modified { id } => {
                let new_config: &ConfigAsset = config_assets.get(id.clone()).unwrap();
                map_config_resource(&mut config, &new_config);

                return;
            }
            _ => (),
        }
    }
}

#[derive(Component, Reflect, Clone, Default)]
pub struct UICamera;

#[derive(Component, Reflect, Clone, Default)]
pub struct OscilCamera;
