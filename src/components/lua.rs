//! Implements loader for a custom asset type.

use bevy::utils::thiserror;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use thiserror::Error;

#[derive(Asset, TypePath, Debug)]
pub struct LuaAsset {
    pub script: String,
}

#[derive(Default)]
pub struct LuaLoader;

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

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CustomAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
}
