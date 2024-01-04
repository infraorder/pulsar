// use bevy::{
//     asset::{AssetServer, Assets, Handle},
//     ecs::{
//         bundle::Bundle,
//         system::{Commands, Res, ResMut},
//     },
//     render::mesh::{shape, Mesh},
//     sprite::Mesh2dHandle,
// };
//
// use crate::{
//     components::{
//         line::{SplitLine, XYLine},
//         lua::LuaAsset,
//     },
//     dsp::{audio_graph::Audio, oscillators::Oscillator, read::Read, Chain, Dsp},
//     InstancingBundle, FREQUENCY_TEMP,
// };
//
// pub struct LuaPulseData {
//     lua_handle: Handle<LuaAsset>,
//     lua_util_handle: Handle<LuaAsset>,
//     lua_string: String,
// }
//
// #[derive(Bundle)]
// pub struct OscillatorBundle {
//     audio: Audio,
//     split_line: SplitLine,
//     xy_line: XYPlotBundle,
// }
//
// #[derive(Bundle)]
// pub struct XYPlotBundle {
//     xy_line: XYLine,
//     instancing: InstancingBundle,
// }
//
// pub fn setup_oscil_temp(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     asset_server: Res<AssetServer>,
// ) {
//     spawn_lua_pulse(&mut commands, asset_server, &mut meshes, true, true);
// }
//
// pub fn spawn_lua_pulse(
//     commands: &mut Commands,
//     asset_server: Res<AssetServer>,
//     meshes: &mut ResMut<Assets<Mesh>>,
//     draw_split: bool,
//     draw_xy: bool,
// ) {
//     let lua_handle: Handle<LuaAsset> =
//         asset_server.load(format!("lua/readers/{}/{}/wave.lua", TYPE.as_str(), NAME));
//     let lua_util_handle: Handle<LuaAsset> =
//         asset_server.load(format!("lua/readers/{}/hood.lua", TYPE.as_str()));
//
//     let split_line = SplitLine {
//         ..Default::default()
//     };
//
//     let mesh = meshes.add(Mesh::from(shape::Circle {
//         radius: 1.0,
//         ..Default::default()
//     }));
//
//     let xy_line = XYPlotBundle {
//         xy_line: XYLine {
//             ..Default::default()
//         },
//         instancing: InstancingBundle {
//             mesh: Mesh2dHandle(mesh),
//             ..Default::default()
//         },
//     };
//
//     let chain = if draw_xy && draw_split {
//         vec![
//             Dsp::Input(Oscillator {
//                 frequency_hz: FREQUENCY_TEMP,
//                 lua_handle,
//                 lua_util_handle,
//                 lua_string: "".to_owned(),
//             }),
//             Dsp::Read(Read {}),
//         ]
//     } else {
//         vec![Dsp::Input(Oscillator {
//             frequency_hz: FREQUENCY_TEMP,
//             lua_handle,
//             lua_util_handle,
//             lua_string: "".to_owned(),
//         })]
//     };
//
//     commands.spawn(Audio::new(Chain { items: chain }));
//
//     if draw_xy {
//         commands.spawn(xy_line);
//     }
//
//     if draw_split {
//         commands.spawn(split_line);
//     }
// }
