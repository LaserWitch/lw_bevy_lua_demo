use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt},
    reflect::TypePath,
};
use bevy_mod_scripting::prelude::CodeAsset;

use anyhow::Error;
use serde::Deserialize;
#[derive(Asset, TypePath, Debug, Deserialize)]
/// A lua code file in bytes
pub struct LuaFennel {
    pub bytes: Vec<u8>,
    pub fennel: bool,
}

impl LuaFennel {
    pub fn source(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(self.bytes.as_slice())
    }
}

impl CodeAsset for LuaFennel {
    fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

#[derive(Default)]
pub struct LuaFennelLoader;

fn _load_fennel(bytes: &[u8]) -> Vec<u8> {
    let file_text = String::from_utf8(bytes.to_vec()).unwrap();
    //Fenel comes with a install() that inserts a searcher,
    //  but we don't actually need that since we're sticking things directly in package.p,
    //  TODO: inline the fennel compiler(big text constant?) rather than depend on it being in the scripts dir.
    let src = format!(
        "return require(\"scripts/fennel\").eval([[ {} ]])",
        file_text
    );
    src.as_bytes().into()
}

fn _load_lua(bytes: &[u8]) -> Vec<u8> {
    bytes.into()
}

impl AssetLoader for LuaFennelLoader {
    type Asset = LuaFennel;
    type Settings = ();
    type Error = Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader, //bytes: &'a [u8],
        _settings: &'a (),
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        //bevy::prelude::info!("lua/fennel loader invoked: {:#}", load_context.asset_path());
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            match load_context.path().extension().map(|s| s.to_str().unwrap()) {
                Some("fnl") => {
                    bevy::prelude::info!("fennel file {:#}", load_context.asset_path());
                    Ok(LuaFennel {
                        bytes,
                        fennel: true,
                    })
                }
                _ => {
                    bevy::prelude::info!("lua file {:#}", load_context.asset_path());
                    Ok(LuaFennel {
                        bytes,
                        fennel: false,
                    })
                }
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        &["lua", "fnl"]
    }
}
