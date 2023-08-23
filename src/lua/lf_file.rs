use bevy::{
    asset::{AssetLoader, Error, LoadedAsset},
    reflect::{TypePath, TypeUuid},
};
use bevy_mod_scripting::prelude::CodeAsset;

use std::sync::Arc;

#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
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
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), Error>> {
        match load_context.path().extension().map(|s| s.to_str().unwrap()) {
            Some("fnl") => {
                let file_text = String::from_utf8(bytes.to_vec()).unwrap();
                //Fenel comes with a install() that inserts a searcher,
                //  but we don't actually need that since we're sticking things directly in package.preload
                //  TODO: inline the fennel compiler(big text constant?) rather than depend on it being in the scripts dir.
                let src = format!(
                    "return require(\"scripts/fennel\").eval([[ {} ]])",
                    file_text
                );
                load_context.set_default_asset(LoadedAsset::new(LuaFennel {
                    bytes: src.as_bytes().into(),
                }));
            }
            _ => {
                load_context.set_default_asset(LoadedAsset::new(LuaFennel {
                    bytes: bytes.into(),
                }));
            }
        }

        Box::pin(async move { Ok(()) })
    }

    fn extensions(&self) -> &[&str] {
        &["lua", "fnl"]
    }
}
