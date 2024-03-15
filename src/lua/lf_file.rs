use bevy::{ asset::{ io::Reader, Asset, AssetLoader, AsyncReadExt }, reflect::TypePath };
use bevy_mod_scripting::prelude::CodeAsset;

use std::sync::Arc;

use anyhow::Error;

#[derive(Debug, Asset, TypePath)]
/// A lua code file in bytes
pub struct LuaFennel {
    pub bytes: Arc<[u8]>,
}

impl CodeAsset for LuaFennel {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Default)]
pub struct LuaFennelLoader;

impl AssetLoader for LuaFennelLoader {
    type Asset = LuaFennel;
    type Settings = ();
    type Error = Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext
    ) -> bevy::asset::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            match
                load_context
                    .path()
                    .extension()
                    .map(|s| s.to_str().unwrap())
            {
                Some("fnl") => {
                    //Fenel comes with a install() that inserts a searcher,
                    //  but we don't actually need that since we're sticking things directly in package.preload
                    //  TODO: inline the fennel compiler(big text constant?) rather than depend on it being in the scripts dir.
                    let code = String::from_utf8(bytes)?;
                    let src = format!("return require(\"scripts/fennel\").eval([[ {} ]])", code);
                    Ok(LuaFennel {
                        bytes: src.as_bytes().into(),
                    })
                }
                _ => {
                    Ok(LuaFennel {
                        bytes: bytes.into(),
                    })
                }
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        &["lua", "fnl"]
    }
}
