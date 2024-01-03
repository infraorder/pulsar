use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, Handle, LoadContext},
    ecs::{component::Component, system::Resource},
    reflect::TypePath,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Asset, TypePath, Debug, Deserialize, Serialize, Resource)]
pub struct ConfigAsset {
    pub width: u32,
    pub height: u32,

    pub line_width: f32,
    pub line_scale_z: f32,
    pub line_scale_y: f32,

    pub line_offset_x_0: f32,
    pub line_offset_y_0: f32,

    pub line_offset_x_1: f32,
    pub line_offset_y_1: f32,

    pub xy_mult: f32,
    pub xy_rad: f32,
}

#[derive(Default, Component)]
pub struct ConfigComp {
    pub handle: Handle<ConfigAsset>,
}

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
}

#[derive(Default)]
pub struct ConfigLoader;

impl AssetLoader for ConfigLoader {
    type Asset = ConfigAsset;
    type Settings = ();
    type Error = CustomAssetLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut script = "".to_string();
            reader.read_to_string(&mut script).await?;
            let config = toml::from_str::<ConfigAsset>(&script).unwrap();

            Ok(config)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CustomAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
}
