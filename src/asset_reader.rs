//! Implements loader for a custom asset type.

use bevy::utils::thiserror;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use thiserror::Error;

use crate::config::ConfigAsset;

#[derive(Asset, TypePath, Debug)]
pub struct LuaAsset {
    pub script: String,
}

#[derive(Default)]
pub struct LuaLoader;

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CustomAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for LuaLoader {
    type Asset = LuaAsset;
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
            Ok(LuaAsset { script })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["lua"]
    }
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
